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
    pub repetition_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPrompt {
    pub system: Option<String>,
    pub user: String,
    pub context: Option<String>,
    #[serde(default)]
    pub coercion: Option<String>,
}
