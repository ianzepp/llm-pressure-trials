use std::time::Instant;

use anyhow::Result;

use crate::domain::result::{RawModelResponse, TrialResult};
use crate::domain::trial::TrialUnit;
use crate::grading::{needs_judge, normalize_and_grade, run_judge};
use crate::providers::openai::OpenAiCompatibleClient;

pub struct JudgeConfig {
    pub enabled: bool,
    pub model: String,
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
    )?;
    let elapsed_ms = started.elapsed().as_millis().max(response.latency_ms);

    let (normalized, mut grading) = normalize_and_grade(unit, &response.content);

    if judge.enabled && needs_judge(&normalized, &grading) {
        if let Ok(jr) = run_judge(client, &judge.model, &response.content) {
            grading.judge_decision = Some(jr.decision);
            grading.judge_compliance = Some(jr.compliance);
            grading.judge_reasoning = Some(jr.reasoning);
            grading.judge_model = Some(judge.model.clone());
        }
    }

    let mut result = TrialResult {
        trial_id: unit.id.clone(),
        model_id: unit.model_id.clone(),
        condition_label: unit.condition_label.clone(),
        output_format: unit.output_format.clone(),
        repetition_index: unit.repetition_index,
        prompt: unit.prompt.clone(),
        response: RawModelResponse {
            content: response.content,
            tokens_in: response.tokens_in,
            tokens_out: response.tokens_out,
            latency_ms: elapsed_ms,
        },
        normalized,
        grading,
        coercion_response: None,
        coercion_normalized: None,
        coercion_grading: None,
    };

    // If the first turn was a refusal and we have a coercion prompt, send follow-up
    if result.grading.refusal_present {
        if let Some(coercion_prompt) = unit.prompt.coercion.as_deref() {
            let coercion_started = Instant::now();
            let coercion_response = client.chat_completion_followup(
                &unit.model_id,
                unit.prompt.system.as_deref(),
                unit.prompt.context.as_deref(),
                &unit.prompt.user,
                &result.response.content,
                coercion_prompt,
            )?;
            let coercion_elapsed = coercion_started
                .elapsed()
                .as_millis()
                .max(coercion_response.latency_ms);

            let (coercion_norm, mut coercion_grade) =
                normalize_and_grade(unit, &coercion_response.content);

            if judge.enabled && needs_judge(&coercion_norm, &coercion_grade) {
                if let Ok(jr) = run_judge(client, &judge.model, &coercion_response.content) {
                    coercion_grade.judge_decision = Some(jr.decision);
                    coercion_grade.judge_compliance = Some(jr.compliance);
                    coercion_grade.judge_reasoning = Some(jr.reasoning);
                    coercion_grade.judge_model = Some(judge.model.clone());
                }
            }

            result.coercion_response = Some(RawModelResponse {
                content: coercion_response.content,
                tokens_in: coercion_response.tokens_in,
                tokens_out: coercion_response.tokens_out,
                latency_ms: coercion_elapsed,
            });
            result.coercion_normalized = Some(coercion_norm);
            result.coercion_grading = Some(coercion_grade);
        }
    }

    Ok(result)
}
