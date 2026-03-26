use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelTarget {
    pub id: String,
    pub label: String,
    pub provider: String,
}

impl std::fmt::Display for ModelTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.provider)
    }
}
