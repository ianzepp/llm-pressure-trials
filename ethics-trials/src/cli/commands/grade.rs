use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::cli::args::GradeArgs;
use crate::domain::result::{RunSummary, TrialResult};
use crate::domain::trial::{ResolvedPrompt, TrialUnit};
use crate::grading::{needs_judge, normalize_and_grade, run_judge};
use crate::loader::studies::StudyLoader;
use crate::providers::openai::OpenAiCompatibleClient;
use crate::runner::compare::compare_results;
use crate::storage::artifacts::ArtifactWriter;
use crate::storage::sqlite::StudyDatabase;

#[derive(Debug, Serialize)]
struct GradeSummary {
    run_id: String,
    study_id: String,
    trials_regraded: usize,
    decisions_changed: usize,
    trials_judged: usize,
    comparisons_regenerated: usize,
}

pub fn run(args: GradeArgs) -> Result<()> {
    let loader = StudyLoader::new(std::env::current_dir()?);
    let study = loader.load_study(&args.study)?;
    let database = StudyDatabase::new(&study.root)?;

    let run_id = match args.run_id {
        Some(id) => id,
        None => database
            .latest_run_id()?
            .ok_or_else(|| anyhow::anyhow!("no runs found for study `{}`", args.study))?,
    };

    let run_dir = study.root.join("runs").join(&run_id);
    let jsonl_path = run_dir.join("trial_results.jsonl");
    if !jsonl_path.exists() {
        anyhow::bail!(
            "trial_results.jsonl not found at {}",
            jsonl_path.display()
        );
    }

    let judge_client = if args.judge {
        Some(OpenAiCompatibleClient::from_env()?)
    } else {
        None
    };

    let file = fs::File::open(&jsonl_path)
        .with_context(|| format!("failed to open {}", jsonl_path.display()))?;
    let reader = BufReader::new(file);

    let mut results: Vec<TrialResult> = Vec::new();
    let mut decisions_changed = 0usize;
    let mut trials_judged = 0usize;

    for line in reader.lines() {
        let line = line.context("failed to read line from trial_results.jsonl")?;
        if line.trim().is_empty() {
            continue;
        }
        let mut result: TrialResult =
            serde_json::from_str(&line).context("failed to parse trial result from JSONL")?;

        let old_decision = result.normalized.decision.clone();

        let stub = TrialUnit {
            id: result.trial_id.clone(),
            study_id: String::new(),
            scenario_id: String::new(),
            experiment_id: None,
            condition_id: None,
            condition_label: result.condition_label.clone(),
            variant_id: None,
            model_id: result.model_id.clone(),
            output_format: result.output_format.clone(),
            prompt: ResolvedPrompt {
                system: None,
                user: String::new(),
                context: None,
                coercion: None,
            },
            decisions: study.scenario.decisions.clone(),
            repetition_index: result.repetition_index,
        };

        let (normalized, mut grading) = normalize_and_grade(&stub, &result.response.content);

        if let Some(client) = &judge_client {
            if needs_judge(&normalized, &grading) {
                if let Ok(jr) = run_judge(client, &args.judge_model, &result.response.content) {
                    grading.judge_decision = Some(jr.decision);
                    grading.judge_compliance = Some(jr.compliance);
                    grading.judge_reasoning = Some(jr.reasoning);
                    grading.judge_model = Some(args.judge_model.clone());
                    trials_judged += 1;
                }
            }
        }

        result.normalized = normalized;
        result.grading = grading;

        if let Some(coercion_resp) = &result.coercion_response {
            let (cn, mut cg) = normalize_and_grade(&stub, &coercion_resp.content);

            if let Some(client) = &judge_client {
                if needs_judge(&cn, &cg) {
                    if let Ok(jr) = run_judge(client, &args.judge_model, &coercion_resp.content) {
                        cg.judge_decision = Some(jr.decision);
                        cg.judge_compliance = Some(jr.compliance);
                        cg.judge_reasoning = Some(jr.reasoning);
                        cg.judge_model = Some(args.judge_model.clone());
                        trials_judged += 1;
                    }
                }
            }

            result.coercion_normalized = Some(cn);
            result.coercion_grading = Some(cg);
        }

        if result.normalized.decision != old_decision {
            decisions_changed += 1;
        }

        database.insert_trial_result(&run_id, &result)?;
        results.push(result);
    }

    // Rewrite the JSONL artifact with regraded data
    let tmp_path = jsonl_path.with_extension("jsonl.tmp");
    {
        let mut out = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .with_context(|| format!("failed to create {}", tmp_path.display()))?;
        for result in &results {
            let line = serde_json::to_string(result)?;
            writeln!(out, "{line}")?;
        }
    }
    fs::rename(&tmp_path, &jsonl_path)
        .with_context(|| format!("failed to rename temp file to {}", jsonl_path.display()))?;

    // Resolve experiment for comparison regeneration
    let selection = database.run_selection(&run_id)?.unwrap_or_default();
    let experiment_id = selection.strip_prefix("experiment:");
    let experiment =
        experiment_id.and_then(|id| study.experiments.iter().find(|item| item.id == id));

    let comparisons = compare_results(experiment, &results);
    database.replace_comparisons(&run_id, &comparisons)?;

    // Write updated comparisons artifact
    let writer = ArtifactWriter::existing(&run_dir)?;
    writer.write_comparisons(&comparisons)?;

    // Update run summary
    let failed_count = database.failed_trial_count(&run_id)?;
    let summary = RunSummary {
        run_id: run_id.clone(),
        study_id: study.id.clone(),
        selection,
        total_trials: results.len() + failed_count,
        successful_trials: results.len(),
        failed_trials: failed_count,
        comparisons: comparisons.len(),
        models: results
            .iter()
            .map(|r| r.model_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect(),
    };
    database.update_run_summary(&summary)?;
    writer.write_summary(&summary)?;

    let grade_summary = GradeSummary {
        run_id,
        study_id: study.id,
        trials_regraded: results.len(),
        decisions_changed,
        trials_judged,
        comparisons_regenerated: comparisons.len(),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&grade_summary)?);
    } else {
        println!("run id: {}", grade_summary.run_id);
        println!("study: {}", grade_summary.study_id);
        println!("trials regraded: {}", grade_summary.trials_regraded);
        println!("decisions changed: {}", grade_summary.decisions_changed);
        println!("trials judged: {}", grade_summary.trials_judged);
        println!(
            "comparisons regenerated: {}",
            grade_summary.comparisons_regenerated
        );
    }

    Ok(())
}
