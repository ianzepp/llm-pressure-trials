use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::domain::model::ModelPricing;
use crate::domain::result::{ComparisonResult, FailedTrial, RunMetadata, RunSummary, TrialResult};
use crate::domain::trial::TrialUnit;

pub struct ArtifactWriter {
    run_dir: PathBuf,
}

impl ArtifactWriter {
    pub fn new(study_root: &Path, run_id: &str) -> Result<Self> {
        let run_dir = study_root.join("runs").join(run_id);
        fs::create_dir_all(&run_dir)
            .with_context(|| format!("failed to create run dir {}", run_dir.display()))?;
        Ok(Self { run_dir })
    }

    pub fn existing(run_dir: &Path) -> Result<Self> {
        anyhow::ensure!(
            run_dir.exists(),
            "run dir does not exist: {}",
            run_dir.display()
        );
        Ok(Self {
            run_dir: run_dir.to_path_buf(),
        })
    }

    pub fn run_dir(&self) -> &Path {
        &self.run_dir
    }

    pub fn write_run_metadata(&self, metadata: &RunMetadata) -> Result<()> {
        write_json(self.run_dir.join("run.json"), metadata)
    }

    pub fn write_trial_units(&self, units: &[TrialUnit]) -> Result<()> {
        write_json(self.run_dir.join("trial_units.json"), units)
    }

    pub fn append_trial_result(&self, result: &TrialResult) -> Result<()> {
        append_jsonl(self.run_dir.join("trial_results.jsonl"), result)
    }

    pub fn append_failed_trial(&self, failure: &FailedTrial) -> Result<()> {
        append_jsonl(self.run_dir.join("trial_failures.jsonl"), failure)
    }

    pub fn write_comparisons(&self, comparisons: &[ComparisonResult]) -> Result<()> {
        write_json(self.run_dir.join("comparisons.json"), comparisons)
    }

    pub fn write_summary(&self, summary: &RunSummary) -> Result<()> {
        write_json(self.run_dir.join("summary.json"), summary)
    }

    pub fn write_pricing(&self, pricing: &[ModelPricing]) -> Result<()> {
        write_json(self.run_dir.join("pricing.json"), pricing)
    }
}

fn write_json<T: serde::Serialize + ?Sized>(path: PathBuf, value: &T) -> Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(&path, data).with_context(|| format!("failed to write {}", path.display()))
}

fn append_jsonl<T: serde::Serialize + ?Sized>(path: PathBuf, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let line = serde_json::to_string(value)?;
    writeln!(file, "{line}").with_context(|| format!("failed to append {}", path.display()))
}
