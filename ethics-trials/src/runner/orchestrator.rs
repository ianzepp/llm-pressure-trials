use anyhow::Result;
use serde::Serialize;

use crate::domain::result::{ComparisonResult, RunMetadata, RunSummary, TrialResult};
use crate::domain::trial::TrialUnit;
use crate::loader::studies::LoadedStudy;
use crate::providers::openai::OpenAiCompatibleClient;
use crate::runner::compare::compare_results;
use crate::runner::execute::execute_trial;
use crate::runner::plan::{expand_experiment, expand_scenario};
use crate::storage::artifacts::ArtifactWriter;
use crate::storage::sqlite::StudyDatabase;

#[derive(Debug, Serialize)]
pub struct DryRunSummary {
    pub study_id: String,
    pub selection: String,
    pub models: Vec<String>,
    pub trial_units: Vec<TrialUnit>,
}

#[derive(Debug, Serialize)]
pub struct LiveRunSummary {
    pub run_id: String,
    pub study_id: String,
    pub selection: String,
    pub models: Vec<String>,
    pub result_count: usize,
    pub comparison_count: usize,
    pub run_dir: std::path::PathBuf,
}

pub fn expand_run_request(
    study: &LoadedStudy,
    scenario_id: Option<&str>,
    experiment_id: Option<&str>,
    models: &[String],
) -> Result<DryRunSummary> {
    let (selection, trial_units) = match (scenario_id, experiment_id) {
        (Some(id), None) => (format!("scenario:{id}"), expand_scenario(study, id, models)?),
        (None, Some(id)) => (format!("experiment:{id}"), expand_experiment(study, id, models)?),
        _ => unreachable!("caller validates mutually exclusive selection"),
    };

    Ok(DryRunSummary {
        study_id: study.id.clone(),
        selection,
        models: models.to_vec(),
        trial_units,
    })
}

pub fn execute_run_request(
    study: &LoadedStudy,
    scenario_id: Option<&str>,
    experiment_id: Option<&str>,
    models: &[String],
) -> Result<LiveRunSummary> {
    let dry_run = expand_run_request(study, scenario_id, experiment_id, models)?;
    let run_id = make_run_id();
    let selection = dry_run.selection.clone();
    let metadata = RunMetadata {
        run_id: run_id.clone(),
        study_id: study.id.clone(),
        selection: selection.clone(),
        models: models.to_vec(),
        created_at_unix_ms: unix_time_ms(),
    };

    let writer = ArtifactWriter::new(&study.root, &run_id)?;
    let database = StudyDatabase::new(&study.root)?;
    writer.write_run_metadata(&metadata)?;
    writer.write_trial_units(&dry_run.trial_units)?;
    database.insert_run_metadata(&metadata)?;

    let client = OpenAiCompatibleClient::from_env()?;
    let mut results: Vec<TrialResult> = Vec::new();

    for unit in &dry_run.trial_units {
        let result = execute_trial(&client, unit)?;
        writer.append_trial_result(&result)?;
        database.insert_trial_result(&run_id, &result)?;
        results.push(result);
    }

    let experiment = experiment_id.and_then(|id| study.experiments.iter().find(|item| item.id == id));
    let comparisons: Vec<ComparisonResult> = compare_results(experiment, &results);
    writer.write_comparisons(&comparisons)?;
    database.replace_comparisons(&run_id, &comparisons)?;

    let summary = RunSummary {
        run_id: run_id.clone(),
        study_id: study.id.clone(),
        selection,
        total_trials: results.len(),
        comparisons: comparisons.len(),
        models: models.to_vec(),
    };
    writer.write_summary(&summary)?;
    database.update_run_summary(&summary)?;

    Ok(LiveRunSummary {
        run_id,
        study_id: study.id.clone(),
        selection: summary.selection,
        models: models.to_vec(),
        result_count: results.len(),
        comparison_count: comparisons.len(),
        run_dir: writer.run_dir().to_path_buf(),
    })
}

fn unix_time_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis()
}

fn make_run_id() -> String {
    format!("run-{}", unix_time_ms())
}
