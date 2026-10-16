use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TrialUnit {
    pub id: String,
    pub study_id: String,
    pub scenario_id: String,
    pub experiment_id: Option<String>,
    pub condition_id: Option<String>,
    pub condition_label: String,
    pub variant_id: Option<String>,
    pub model_id: String,
    pub output_format: Option<String>,
    pub prompt: ResolvedPrompt,
    pub decisions: Vec<String>,
    pub repetition_index: u32,
    #[serde(default)]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPrompt {
    pub system: Option<String>,
    pub user: String,
    pub context: Option<String>,
    #[serde(default)]
    pub coercion: Option<String>,
    /// Follow-up sent when the model *complied* (opposite of coercion).
    /// Used for two-stage escalation: get agreement in stage 1, then
    /// ask the model to execute on that agreement in stage 2.
    #[serde(default)]
    pub escalation: Option<String>,
}
