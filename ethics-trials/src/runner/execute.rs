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

    Ok(TrialResult {
        trial_id: unit.id.clone(),
        model_id: unit.model_id.clone(),
        condition_label: unit.condition_label.clone(),
        output_format: unit.output_format.clone(),
        prompt: unit.prompt.clone(),
        response: RawModelResponse {
            content: response.content,
            tokens_in: response.tokens_in,
            tokens_out: response.tokens_out,
            latency_ms: elapsed_ms,
        },
        normalized,
        grading,
    })
}
