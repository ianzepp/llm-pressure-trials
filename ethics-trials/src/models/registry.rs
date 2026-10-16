use crate::domain::model::ModelTarget;

pub fn built_in_models() -> Vec<ModelTarget> {
    vec![
        ModelTarget {
            id: "openai/gpt-4o-mini".to_owned(),
            label: "gpt-4o-mini".to_owned(),
            provider: "openrouter".to_owned(),
        },
        ModelTarget {
            id: "openai/gpt-4.1-mini".to_owned(),
            label: "gpt-4.1-mini".to_owned(),
            provider: "openrouter".to_owned(),
        },
        ModelTarget {
            id: "anthropic/claude-3.5-haiku".to_owned(),
            label: "claude-3.5-haiku".to_owned(),
            provider: "openrouter".to_owned(),
        },
    ]
}
