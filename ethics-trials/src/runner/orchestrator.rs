use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::json;

use crate::domain::result::{ComparisonResult, FailedTrial, RunMetadata, RunSummary, TrialResult};
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
    pub failed_count: usize,
    pub comparison_count: usize,
    pub run_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct SchedulerConfig {
    pub launch_interval_ms: u64,
    pub high_water: usize,
    pub low_water: usize,
}

pub fn expand_run_request(
    study: &LoadedStudy,
    scenario_id: Option<&str>,
    experiment_id: Option<&str>,
    models: &[String],
) -> Result<DryRunSummary> {
    let (selection, trial_units) = match (scenario_id, experiment_id) {
        (Some(id), None) => (
            format!("scenario:{id}"),
            expand_scenario(study, id, models)?,
        ),
        (None, Some(id)) => (
            format!("experiment:{id}"),
            expand_experiment(study, id, models)?,
        ),
        _ => bail!("exactly one of scenario_id or experiment_id must be set"),
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
    scheduler: SchedulerConfig,
    jsonl: bool,
    continue_on_error: bool,
) -> Result<LiveRunSummary> {
    let dry_run = expand_run_request(study, scenario_id, experiment_id, models)?;
    let run_id = make_run_id()?;
    let selection = dry_run.selection.clone();
    let metadata = RunMetadata {
        run_id: run_id.clone(),
        study_id: study.id.clone(),
        selection: selection.clone(),
        models: models.to_vec(),
        created_at_unix_ms: unix_time_ms()?,
    };

    let writer = ArtifactWriter::new(&study.root, &run_id)?;
    let database = StudyDatabase::new(&study.root)?;
    writer.write_run_metadata(&metadata)?;
    writer.write_trial_units(&dry_run.trial_units)?;
    database.insert_run_metadata(&metadata)?;

    let client = OpenAiCompatibleClient::from_env()?;
    let mut results: Vec<TrialResult> = Vec::new();
    let mut failures: Vec<FailedTrial> = Vec::new();
    let grouped_units = group_trial_units(&dry_run.trial_units);
    let mut sequences = grouped_units
        .into_iter()
        .map(VecDeque::from)
        .collect::<Vec<VecDeque<crate::domain::trial::TrialUnit>>>();
    let mut ready = (0..sequences.len()).collect::<VecDeque<_>>();
    let mut inflight = 0usize;
    let total_trials = dry_run.trial_units.len();
    let mut completed = 0usize;
    let mut backpressured = false;
    let mut last_launch: Option<Instant> = None;
    let (tx, rx) = mpsc::channel::<WorkerMessage>();

    emit_jsonl_event(
        jsonl,
        json!({
            "ts_unix_ms": unix_time_ms()?,
            "event": "run_started",
            "run_id": run_id,
            "study_id": study.id,
            "selection": selection,
            "model_count": models.len(),
            "trial_count": total_trials,
            "launch_interval_ms": scheduler.launch_interval_ms,
            "high_water": scheduler.high_water,
            "low_water": scheduler.low_water,
        }),
    )?;

    while completed < total_trials {
        while let Ok(message) = rx.try_recv() {
            inflight = inflight.saturating_sub(1);
            match message.result {
                Ok(result) => {
                    emit_jsonl_event(
                        jsonl,
                        json!({
                            "ts_unix_ms": unix_time_ms()?,
                            "event": "trial_completed",
                            "run_id": run_id,
                            "study_id": study.id,
                            "trial_id": result.trial_id,
                            "model_id": result.model_id,
                            "condition_label": result.condition_label,
                            "latency_ms": result.response.latency_ms,
                            "format_valid": result.grading.format_valid,
                            "decision": result.normalized.decision,
                            "compliance_level": result.grading.compliance_level,
                            "refusal_present": result.grading.refusal_present,
                            "in_flight": inflight,
                            "completed": completed + 1,
                            "total_trials": total_trials,
                        }),
                    )?;
                    writer.append_trial_result(&result)?;
                    database.insert_trial_result(&run_id, &result)?;
                    results.push(result);
                }
                Err(error) => {
                    let failure = FailedTrial {
                        trial_id: message.trial_id.clone(),
                        model_id: message.model_id.clone(),
                        condition_label: message.condition_label.clone(),
                        error: format!("{error:#}"),
                    };
                    emit_jsonl_event(
                        jsonl,
                        json!({
                            "ts_unix_ms": unix_time_ms()?,
                            "event": "trial_failed",
                            "run_id": run_id,
                            "study_id": study.id,
                            "trial_id": message.trial_id,
                            "model_id": message.model_id,
                            "condition_label": message.condition_label,
                            "in_flight": inflight,
                            "error": failure.error,
                        }),
                    )?;
                    writer.append_failed_trial(&failure)?;
                    database.insert_failed_trial(&run_id, &failure)?;
                    failures.push(failure);
                    if !continue_on_error {
                        return Err(error);
                    }
                }
            }
            completed += 1;

            if !sequences[message.sequence_index].is_empty() {
                ready.push_back(message.sequence_index);
            }

            if backpressured && inflight <= scheduler.low_water {
                backpressured = false;
                emit_jsonl_event(
                    jsonl,
                    json!({
                        "ts_unix_ms": unix_time_ms()?,
                        "event": "backpressure_off",
                        "run_id": run_id,
                        "study_id": study.id,
                        "in_flight": inflight,
                        "high_water": scheduler.high_water,
                        "low_water": scheduler.low_water,
                    }),
                )?;
            }
        }

        if inflight >= scheduler.high_water && !backpressured {
            backpressured = true;
            emit_jsonl_event(
                jsonl,
                json!({
                    "ts_unix_ms": unix_time_ms()?,
                    "event": "backpressure_on",
                    "run_id": run_id,
                    "study_id": study.id,
                    "in_flight": inflight,
                    "high_water": scheduler.high_water,
                    "low_water": scheduler.low_water,
                }),
            )?;
        }

        let launch_due = last_launch.is_none_or(|last| {
            last.elapsed() >= Duration::from_millis(scheduler.launch_interval_ms)
        });

        if !backpressured
            && inflight < scheduler.high_water
            && launch_due
            && let Some(sequence_index) = ready.pop_front()
            && let Some(unit) = sequences[sequence_index].pop_front()
        {
            let tx = tx.clone();
            let client = client.clone();
            let launched_trial_id = unit.id.clone();
            let launched_model_id = unit.model_id.clone();
            let launched_condition_label = unit.condition_label.clone();
            inflight += 1;
            last_launch = Some(Instant::now());

            emit_jsonl_event(
                jsonl,
                json!({
                    "ts_unix_ms": unix_time_ms()?,
                    "event": "trial_launched",
                    "run_id": run_id,
                    "study_id": study.id,
                    "trial_id": launched_trial_id,
                    "model_id": launched_model_id,
                    "condition_label": launched_condition_label,
                    "in_flight": inflight,
                    "high_water": scheduler.high_water,
                    "low_water": scheduler.low_water,
                    "launch_interval_ms": scheduler.launch_interval_ms,
                }),
            )?;

            std::thread::spawn(move || {
                let result = execute_trial(&client, &unit);
                send_worker_message(
                    &tx,
                    WorkerMessage {
                        sequence_index,
                        trial_id: unit.id.clone(),
                        model_id: unit.model_id.clone(),
                        condition_label: unit.condition_label.clone(),
                        result,
                    },
                );
            });
            continue;
        }

        if completed == total_trials {
            break;
        }

        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(message) => {
                inflight = inflight.saturating_sub(1);
                match message.result {
                    Ok(result) => {
                        emit_jsonl_event(
                            jsonl,
                            json!({
                                "ts_unix_ms": unix_time_ms()?,
                                "event": "trial_completed",
                                "run_id": run_id,
                                "study_id": study.id,
                                "trial_id": result.trial_id,
                                "model_id": result.model_id,
                                "condition_label": result.condition_label,
                                "latency_ms": result.response.latency_ms,
                                "format_valid": result.grading.format_valid,
                                "decision": result.normalized.decision,
                                "compliance_level": result.grading.compliance_level,
                                "refusal_present": result.grading.refusal_present,
                                "in_flight": inflight,
                                "completed": completed + 1,
                                "total_trials": total_trials,
                            }),
                        )?;
                        writer.append_trial_result(&result)?;
                        database.insert_trial_result(&run_id, &result)?;
                        results.push(result);
                    }
                    Err(error) => {
                        let failure = FailedTrial {
                            trial_id: message.trial_id.clone(),
                            model_id: message.model_id.clone(),
                            condition_label: message.condition_label.clone(),
                            error: format!("{error:#}"),
                        };
                        emit_jsonl_event(
                            jsonl,
                            json!({
                                "ts_unix_ms": unix_time_ms()?,
                                "event": "trial_failed",
                                "run_id": run_id,
                                "study_id": study.id,
                                "trial_id": message.trial_id,
                                "model_id": message.model_id,
                                "condition_label": message.condition_label,
                                "in_flight": inflight,
                                "error": failure.error,
                            }),
                        )?;
                        writer.append_failed_trial(&failure)?;
                        database.insert_failed_trial(&run_id, &failure)?;
                        failures.push(failure);
                        if !continue_on_error {
                            return Err(error);
                        }
                    }
                }
                completed += 1;

                if !sequences[message.sequence_index].is_empty() {
                    ready.push_back(message.sequence_index);
                }

                if backpressured && inflight <= scheduler.low_water {
                    backpressured = false;
                    emit_jsonl_event(
                        jsonl,
                        json!({
                            "ts_unix_ms": unix_time_ms()?,
                            "event": "backpressure_off",
                            "run_id": run_id,
                            "study_id": study.id,
                            "in_flight": inflight,
                            "high_water": scheduler.high_water,
                            "low_water": scheduler.low_water,
                        }),
                    )?;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if completed < total_trials {
                    return Err(anyhow::anyhow!(
                        "scheduler channel disconnected before all trials completed"
                    ));
                }
            }
        }
    }

    let experiment =
        experiment_id.and_then(|id| study.experiments.iter().find(|item| item.id == id));
    let comparisons: Vec<ComparisonResult> = compare_results(experiment, &results);
    writer.write_comparisons(&comparisons)?;
    database.replace_comparisons(&run_id, &comparisons)?;

    let summary = RunSummary {
        run_id: run_id.clone(),
        study_id: study.id.clone(),
        selection,
        total_trials,
        successful_trials: results.len(),
        failed_trials: failures.len(),
        comparisons: comparisons.len(),
        models: models.to_vec(),
    };
    writer.write_summary(&summary)?;
    database.update_run_summary(&summary)?;

    emit_jsonl_event(
        jsonl,
        json!({
            "ts_unix_ms": unix_time_ms()?,
            "event": "run_completed",
            "run_id": run_id,
            "study_id": study.id,
            "selection": summary.selection,
            "result_count": results.len(),
            "failed_count": failures.len(),
            "comparison_count": comparisons.len(),
            "run_dir": writer.run_dir().display().to_string(),
        }),
    )?;

    Ok(LiveRunSummary {
        run_id,
        study_id: study.id.clone(),
        selection: summary.selection,
        models: models.to_vec(),
        result_count: results.len(),
        failed_count: failures.len(),
        comparison_count: comparisons.len(),
        run_dir: writer.run_dir().to_path_buf(),
    })
}

struct WorkerMessage {
    sequence_index: usize,
    trial_id: String,
    model_id: String,
    condition_label: String,
    result: Result<TrialResult>,
}

fn group_trial_units(
    units: &[crate::domain::trial::TrialUnit],
) -> Vec<Vec<crate::domain::trial::TrialUnit>> {
    let mut grouped = Vec::new();
    let mut order = Vec::new();
    let mut by_model = HashMap::<String, Vec<crate::domain::trial::TrialUnit>>::new();

    for unit in units {
        if !by_model.contains_key(&unit.model_id) {
            order.push(unit.model_id.clone());
        }
        by_model
            .entry(unit.model_id.clone())
            .or_default()
            .push(unit.clone());
    }

    for model_id in order {
        if let Some(group) = by_model.remove(&model_id) {
            grouped.push(group);
        }
    }

    grouped
}

fn unix_time_ms() -> Result<u128> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system clock is set before the unix epoch")
        .map(|duration| duration.as_millis())
}

fn make_run_id() -> Result<String> {
    Ok(format!("run-{}", unix_time_ms()?))
}

fn send_worker_message(tx: &mpsc::Sender<WorkerMessage>, message: WorkerMessage) {
    if tx.send(message).is_err() {
        // The scheduler can drop the receiver when it exits early after a fatal trial failure.
    }
}

fn emit_jsonl_event(enabled: bool, value: serde_json::Value) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &value)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
#[path = "orchestrator_test.rs"]
mod tests;
