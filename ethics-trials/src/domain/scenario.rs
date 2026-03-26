use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDefinition {
    pub id: String,
    pub title: String,
    pub family: String,
    pub description: String,
    #[serde(default)]
    pub hypothesis: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub risk_level: Option<String>,
    #[serde(default)]
    pub response_mode: Option<String>,
    #[serde(default)]
    pub output_formats: Vec<String>,
    pub prompt: PromptDefinition,
    #[serde(default)]
    pub variants: Vec<ScenarioVariant>,
    #[serde(default)]
    pub conditions: Vec<ScenarioCondition>,
    #[serde(default)]
    pub attached_rubrics: Vec<String>,
    #[serde(default)]
    pub metadata: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDefinition {
    #[serde(default)]
    pub system: Option<String>,
    pub user: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub coercion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioVariant {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub changes: VariantChanges,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariantChanges {
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub prompt_suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioCondition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub prompt_overrides: serde_yaml::Value,
}
