use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ModelTarget {
    pub id: String,
    pub label: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_id: String,
    pub prompt_cost_per_token: f64,
    pub completion_cost_per_token: f64,
}

impl std::fmt::Display for ModelTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.provider)
    }
}
