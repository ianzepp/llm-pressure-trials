use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::domain::model::ModelTarget;

use crate::providers::openai::OpenAiCompatibleClient;

const CACHE_DIR: &str = ".cache/openrouter";
const CACHE_FILE: &str = "models.json";

#[derive(Debug, Serialize)]
pub struct CacheSyncSummary {
    pub cache_path: PathBuf,
    pub model_count: usize,
    pub synced_at_unix_ms: u128,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedModelsFile {
    synced_at_unix_ms: u128,
    source: String,
    model_count: usize,
    data: Vec<OpenRouterModelCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterModelCacheEntry {
    id: String,
    canonical_slug: Option<String>,
    name: Option<String>,
    created: Option<i64>,
    description: Option<String>,
    context_length: Option<u64>,
    architecture: serde_json::Value,
    pricing: serde_json::Value,
    top_provider: serde_json::Value,
    per_request_limits: serde_json::Value,
}

pub fn write_openrouter_models_cache(
    root: &Path,
    client: &OpenAiCompatibleClient,
) -> Result<CacheSyncSummary> {
    let response = client.list_models()?;
    let synced_at_unix_ms = unix_time_ms()?;
    let cache_path = root.join(CACHE_DIR).join(CACHE_FILE);

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {}", parent.display()))?;
    }

    let cached = CachedModelsFile {
        synced_at_unix_ms,
        source: "https://openrouter.ai/api/v1/models".to_owned(),
        model_count: response.data.len(),
        data: response
            .data
            .into_iter()
            .map(|model| OpenRouterModelCacheEntry {
                id: model.id,
                canonical_slug: model.canonical_slug,
                name: model.name,
                created: model.created,
                description: model.description,
                context_length: model.context_length,
                architecture: model.architecture.unwrap_or(serde_json::Value::Null),
                pricing: model.pricing.unwrap_or(serde_json::Value::Null),
                top_provider: model.top_provider.unwrap_or(serde_json::Value::Null),
                per_request_limits: model.per_request_limits.unwrap_or(serde_json::Value::Null),
            })
            .collect(),
    };

    let bytes = serde_json::to_vec_pretty(&cached)?;
    fs::write(&cache_path, bytes)
        .with_context(|| format!("failed to write cache file {}", cache_path.display()))?;

    Ok(CacheSyncSummary {
        cache_path,
        model_count: cached.model_count,
        synced_at_unix_ms,
    })
}

pub fn load_cached_models(root: &Path) -> Result<Vec<ModelTarget>> {
    let cache_path = root.join(CACHE_DIR).join(CACHE_FILE);
    let bytes = fs::read(&cache_path)
        .with_context(|| format!("failed to read cache file {}", cache_path.display()))?;
    let cached: CachedModelsFile = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse cache file {}", cache_path.display()))?;

    Ok(cached
        .data
        .into_iter()
        .map(|item| ModelTarget {
            label: item.name.unwrap_or_else(|| item.id.clone()),
            id: item.id,
            provider: "openrouter".to_owned(),
        })
        .collect())
}

fn unix_time_ms() -> Result<u128> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system clock is set before the unix epoch")
        .map(|duration| duration.as_millis())
}
