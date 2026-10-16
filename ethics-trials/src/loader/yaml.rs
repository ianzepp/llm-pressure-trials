use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

pub fn load_yaml_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML file {}", path.display()))?;
    serde_yaml::from_str(&contents)
        .with_context(|| format!("failed to parse YAML file {}", path.display()))
}
