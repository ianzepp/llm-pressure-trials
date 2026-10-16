use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricDefinition {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub applies_to: Option<RubricAppliesTo>,
    pub layers: Vec<RubricLayer>,
    #[serde(default)]
    pub signals: Vec<RubricSignal>,
    #[serde(default)]
    pub metadata: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricAppliesTo {
    #[serde(default)]
    pub families: Vec<String>,
    #[serde(default)]
    pub response_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricLayer {
    pub id: String,
    #[serde(rename = "type")]
    pub layer_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rules: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricSignal {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub signal_type: String,
    #[serde(default)]
    pub description: Option<String>,
}
