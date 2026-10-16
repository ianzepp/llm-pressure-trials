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
    pub decisions: Vec<String>,
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
    /// Follow-up sent when the model complied (opposite of coercion).
    /// Used for two-stage escalation: elicit agreement, then demand action.
    #[serde(default)]
    pub escalation: Option<String>,
    /// Alternating coercion texts. When non-empty this supersedes `coercion`:
    /// the variant is chosen by the unit's ordinal among its model's trial
    /// units, modulo the number of variants.
    #[serde(default)]
    pub coercion_variants: Vec<CoercionVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoercionVariant {
    pub id: String,
    pub text: String,
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
    /// Always-sent second user turn. When set, the scenario-level `coercion`,
    /// `coercion_variants` and `escalation` are not sent for this variant.
    #[serde(default)]
    pub followup: Option<String>,
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
