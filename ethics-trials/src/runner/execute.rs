use std::time::Instant;

use anyhow::Result;

use crate::domain::result::{RawModelResponse, TrialResult};
use crate::domain::trial::TrialUnit;
use crate::grading::normalize_and_grade;
use crate::providers::openai::OpenAiCompatibleClient;

pub fn execute_trial(client: &OpenAiCompatibleClient, unit: &TrialUnit) -> Result<TrialResult> {
    let started = Instant::now();
    let response = client.chat_completion(
        &unit.model_id,
        unit.prompt.system.as_deref(),
        unit.prompt.context.as_deref(),
        &unit.prompt.user,
    )?;
    let elapsed_ms = started.elapsed().as_millis().max(response.latency_ms);

    let (normalized, grading) = normalize_and_grade(unit, &response.content);

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

            let (coercion_norm, coercion_grade) =
                normalize_and_grade(unit, &coercion_response.content);

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
