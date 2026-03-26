use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::domain::experiment::ExperimentDefinition;
use crate::domain::rubric::RubricDefinition;
use crate::domain::scenario::ScenarioDefinition;
use crate::loader::yaml::load_yaml_file;

#[derive(Debug, Clone)]
pub struct LoadedStudy {
    pub id: String,
    #[allow(dead_code)]
    pub root: PathBuf,
    pub scenario: ScenarioDefinition,
    pub experiments: Vec<ExperimentDefinition>,
    pub rubrics: Vec<RubricDefinition>,
}

pub struct StudyLoader {
    root: PathBuf,
}

impl StudyLoader {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn list_studies(&self) -> Result<Vec<String>> {
        let studies_dir = self.root.join("studies");
        let mut studies = Vec::new();

        for entry in fs::read_dir(&studies_dir)
            .with_context(|| format!("failed to read {}", studies_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                studies.push(entry.file_name().to_string_lossy().into_owned());
            }
        }

        studies.sort();
        Ok(studies)
    }

    pub fn load_study(&self, study_id: &str) -> Result<LoadedStudy> {
        let root = self.root.join("studies").join(study_id);
        let scenario = load_yaml_file::<ScenarioDefinition>(&root.join("scenario.yml"))?;
        let experiments = load_definitions::<ExperimentDefinition>(&root.join("experiments"))?;
        let rubrics = load_definitions::<RubricDefinition>(&root.join("rubrics"))?;

        Ok(LoadedStudy {
            id: study_id.to_owned(),
            root,
            scenario,
            experiments,
            rubrics,
        })
    }
}

fn load_definitions<T>(dir: &Path) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();

    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|item| item.to_str()) == Some("yml") {
            items.push(load_yaml_file::<T>(&path)?);
        }
    }

    Ok(items)
}
