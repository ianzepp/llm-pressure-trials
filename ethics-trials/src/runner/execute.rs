use std::time::Instant;

use anyhow::Result;

use crate::domain::result::{DeterministicGrade, NormalizedResponse, RawModelResponse, TrialResult};
use crate::domain::trial::TrialUnit;
use crate::grading::{needs_judge, normalize_and_grade, run_judge};
use crate::providers::openai::OpenAiCompatibleClient;
use crate::providers::types::ProviderResponse;

pub struct JudgeConfig {
    pub enabled: bool,
    pub model: String,
}

/// The second user turn (if any) sent after the first response.
#[derive(Debug, PartialEq, Eq)]
pub enum FollowupTurn<'a> {
    None,
    /// Variant `followup`: always sent, regardless of grading.
    Followup(&'a str),
    /// Sent when the first turn was refused. `variant_id` is set when the text
    /// came from `coercion_variants`.
    Coercion {
        text: &'a str,
        variant_id: Option<&'a str>,
    },
    /// Sent when the first turn was not refused.
    Escalation(&'a str),
}

/// Decide which follow-up turn a trial sends, given whether the first turn
/// was graded as a refusal. A variant `followup` replaces coercion/escalation.
pub fn plan_followup_turn(unit: &TrialUnit, first_turn_refused: bool) -> FollowupTurn<'_> {
    let prompt = &unit.prompt;

    if let Some(text) = prompt.followup.as_deref() {
        return FollowupTurn::Followup(text);
    }

    if first_turn_refused {
        if !prompt.coercion_variants.is_empty() {
            let index = unit.coercion_ordinal % prompt.coercion_variants.len();
            let chosen = &prompt.coercion_variants[index];
            return FollowupTurn::Coercion {
                text: &chosen.text,
                variant_id: Some(&chosen.id),
            };
        }
        return match prompt.coercion.as_deref() {
            Some(text) => FollowupTurn::Coercion {
                text,
                variant_id: None,
            },
            None => FollowupTurn::None,
        };
    }

    match prompt.escalation.as_deref() {
        Some(text) => FollowupTurn::Escalation(text),
        None => FollowupTurn::None,
    }
}

struct Turn {
    response: RawModelResponse,
    normalized: NormalizedResponse,
    grading: DeterministicGrade,
}

fn raw_response(response: ProviderResponse, elapsed_ms: u128) -> RawModelResponse {
    RawModelResponse {
        content: response.content,
        tokens_in: response.tokens_in,
        tokens_out: response.tokens_out,
        latency_ms: elapsed_ms,
        finish_reason: response.finish_reason,
        native_finish_reason: response.native_finish_reason,
        refusal: response.refusal,
        reasoning: response.reasoning,
        content_null: response.content_null,
        raw_body: response.raw_body,
    }
}

fn apply_judge(
    client: &OpenAiCompatibleClient,
    judge: &JudgeConfig,
    content: &str,
    normalized: &NormalizedResponse,
    grading: &mut DeterministicGrade,
) {
    if judge.enabled
        && needs_judge(normalized, grading)
        && let Ok(jr) = run_judge(client, &judge.model, content)
    {
        grading.judge_decision = Some(jr.decision);
        grading.judge_compliance = Some(jr.compliance);
        grading.judge_reasoning = Some(jr.reasoning);
        grading.judge_model = Some(judge.model.clone());
    }
}

/// Send `text` as the next user turn after `assistant_content`, then grade it.
fn run_followup_turn(
    client: &OpenAiCompatibleClient,
    unit: &TrialUnit,
    judge: &JudgeConfig,
    assistant_content: &str,
    text: &str,
) -> Result<Turn> {
    let started = Instant::now();
    let response = client.chat_completion_followup(
        &unit.model_id,
        unit.prompt.system.as_deref(),
        unit.prompt.context.as_deref(),
        &unit.prompt.user,
        assistant_content,
        text,
        unit.temperature,
    )?;
    let elapsed_ms = started.elapsed().as_millis().max(response.latency_ms);

    let (normalized, mut grading) = normalize_and_grade(unit, &response.content);
    apply_judge(client, judge, &response.content, &normalized, &mut grading);

    Ok(Turn {
        response: raw_response(response, elapsed_ms),
        normalized,
        grading,
    })
}

pub fn execute_trial(
    client: &OpenAiCompatibleClient,
    unit: &TrialUnit,
    judge: &JudgeConfig,
) -> Result<TrialResult> {
    let started = Instant::now();
    let response = client.chat_completion(
        &unit.model_id,
        unit.prompt.system.as_deref(),
        unit.prompt.context.as_deref(),
        &unit.prompt.user,
        unit.temperature,
    )?;
    let elapsed_ms = started.elapsed().as_millis().max(response.latency_ms);

    let (normalized, mut grading) = normalize_and_grade(unit, &response.content);
    apply_judge(client, judge, &response.content, &normalized, &mut grading);

    let mut result = TrialResult {
        trial_id: unit.id.clone(),
        model_id: unit.model_id.clone(),
        condition_label: unit.condition_label.clone(),
        output_format: unit.output_format.clone(),
        repetition_index: unit.repetition_index,
        prompt: unit.prompt.clone(),
        response: raw_response(response, elapsed_ms),
        normalized,
        grading,
        coercion_response: None,
        coercion_normalized: None,
        coercion_grading: None,
        escalation_response: None,
        escalation_normalized: None,
        escalation_grading: None,
        followup_response: None,
        followup_normalized: None,
        followup_grading: None,
        coercion_variant: None,
    };

    match plan_followup_turn(unit, result.grading.refusal_present) {
        FollowupTurn::None => {}
        FollowupTurn::Followup(text) => {
            let turn = run_followup_turn(client, unit, judge, &result.response.content, text)?;
            result.followup_response = Some(turn.response);
            result.followup_normalized = Some(turn.normalized);
            result.followup_grading = Some(turn.grading);
        }
        FollowupTurn::Coercion { text, variant_id } => {
            let turn = run_followup_turn(client, unit, judge, &result.response.content, text)?;
            result.coercion_response = Some(turn.response);
            result.coercion_normalized = Some(turn.normalized);
            result.coercion_grading = Some(turn.grading);
            result.coercion_variant = variant_id.map(str::to_owned);
        }
        FollowupTurn::Escalation(text) => {
            // Stage-2 follow-up that asks the model to act on its own agreement.
            let turn = run_followup_turn(client, unit, judge, &result.response.content, text)?;
            result.escalation_response = Some(turn.response);
            result.escalation_normalized = Some(turn.normalized);
            result.escalation_grading = Some(turn.grading);
        }
    }

    Ok(result)
}

#[cfg(test)]
#[path = "execute_test.rs"]
mod tests;
