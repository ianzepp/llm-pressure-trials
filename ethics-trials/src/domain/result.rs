use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialResult {
    pub trial_id: String,
    pub model_id: String,
    pub condition_label: String,
    pub output_format: Option<String>,
    pub repetition_index: u32,
    pub prompt: super::trial::ResolvedPrompt,
    pub response: RawModelResponse,
    pub normalized: NormalizedResponse,
    pub grading: DeterministicGrade,
    #[serde(default)]
    pub coercion_response: Option<RawModelResponse>,
    #[serde(default)]
    pub coercion_normalized: Option<NormalizedResponse>,
    #[serde(default)]
    pub coercion_grading: Option<DeterministicGrade>,
    /// Stage-2 escalation: follow-up sent when the model complied in stage 1.
    #[serde(default)]
    pub escalation_response: Option<RawModelResponse>,
    #[serde(default)]
    pub escalation_normalized: Option<NormalizedResponse>,
    #[serde(default)]
    pub escalation_grading: Option<DeterministicGrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTrial {
    pub trial_id: String,
    pub model_id: String,
    pub condition_label: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawModelResponse {
    pub content: String,
    pub tokens_in: Option<u32>,
    pub tokens_out: Option<u32>,
    pub latency_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedResponse {
    pub raw_text: String,
    pub parsed_json: Option<serde_json::Value>,
    pub decision: Option<String>,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicGrade {
    pub format_valid: bool,
    pub refusal_present: bool,
    pub compliance_level: String,
    pub harmful_specificity: u8,
    pub hedging_level: u8,
    pub notes: Vec<String>,
    #[serde(default)]
    pub judge_decision: Option<String>,
    #[serde(default)]
    pub judge_compliance: Option<String>,
    #[serde(default)]
    pub judge_reasoning: Option<String>,
    #[serde(default)]
    pub judge_model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResult {
    pub model_id: String,
    pub comparison_id: String,
    pub left_label: String,
    pub right_label: String,
    pub metric: Option<String>,
    pub left_decision: Option<String>,
    pub right_decision: Option<String>,
    pub decision_shift: bool,
    pub refusal_shift: bool,
    pub hedging_delta: i16,
    pub specificity_delta: i16,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunMetadata {
    pub run_id: String,
    pub study_id: String,
    pub selection: String,
    pub models: Vec<String>,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub study_id: String,
    pub selection: String,
    pub total_trials: usize,
    pub successful_trials: usize,
    pub failed_trials: usize,
    pub comparisons: usize,
    pub models: Vec<String>,
}
