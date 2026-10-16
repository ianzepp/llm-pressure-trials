use crate::domain::model::ModelTarget;

pub fn parse_providers(input: Option<&str>) -> Option<Vec<String>> {
    let providers = input?
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| item.to_owned())
        .collect::<Vec<_>>();

    if providers.is_empty() {
        None
    } else {
        Some(providers)
    }
}

pub fn filter_models(
    models: Vec<ModelTarget>,
    pattern: Option<&str>,
    providers: Option<&[String]>,
    limit: Option<usize>,
) -> Vec<ModelTarget> {
    let mut models = models
        .into_iter()
        .filter(|item| pattern.is_none_or(|pattern| matches_pattern(&item.id, pattern)))
        .filter(|item| providers.is_none_or(|providers| provider_allowed(&item.id, providers)))
        .collect::<Vec<_>>();

    if let Some(limit) = limit {
        models.truncate(limit);
    }

    models
}

pub fn filter_model_ids(
    model_ids: Vec<String>,
    providers: Option<&[String]>,
    limit: Option<usize>,
) -> Vec<String> {
    let mut model_ids = model_ids
        .into_iter()
        .filter(|item| providers.is_none_or(|providers| provider_allowed(item, providers)))
        .collect::<Vec<_>>();

    if let Some(limit) = limit {
        model_ids.truncate(limit);
    }

    model_ids
}

fn provider_allowed(model_id: &str, providers: &[String]) -> bool {
    let provider = model_id.split('/').next().unwrap_or_default();
    providers.iter().any(|item| item == provider)
}

fn matches_pattern(value: &str, pattern: &str) -> bool {
    let value = value.as_bytes();
    let pattern = pattern.as_bytes();
    let mut dp = vec![vec![false; pattern.len() + 1]; value.len() + 1];
    dp[0][0] = true;

    for j in 1..=pattern.len() {
        if pattern[j - 1] == b'*' {
            dp[0][j] = dp[0][j - 1];
        }
    }

    for i in 1..=value.len() {
        for j in 1..=pattern.len() {
            dp[i][j] = match pattern[j - 1] {
                b'*' => dp[i][j - 1] || dp[i - 1][j],
                b'?' => dp[i - 1][j - 1],
                byte => dp[i - 1][j - 1] && value[i - 1] == byte,
            };
        }
    }

    dp[value.len()][pattern.len()]
}
