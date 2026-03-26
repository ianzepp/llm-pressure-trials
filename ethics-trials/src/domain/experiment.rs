use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentDefinition {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub mode: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_repeats")]
    pub repeats: u32,
    pub units: Vec<ExperimentUnit>,
    #[serde(default)]
    pub comparisons: Vec<ExperimentComparison>,
    #[serde(default)]
    pub attached_rubrics: Vec<String>,
    #[serde(default)]
    pub metadata: serde_yaml::Value,
}

fn default_repeats() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentUnit {
    pub label: String,
    pub scenario: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub repeat: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentComparison {
    pub id: String,
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub metric: Option<String>,
}
