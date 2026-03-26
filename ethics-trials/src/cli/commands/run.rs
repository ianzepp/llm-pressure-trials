use anyhow::{Result, bail};

use crate::cli::args::RunArgs;
use crate::loader::studies::StudyLoader;
use crate::loader::validate::validate_study;
use crate::models::registry;
use crate::runner::orchestrator::{
    DryRunSummary, LiveRunSummary, expand_run_request, execute_run_request,
};

pub fn run(args: RunArgs) -> Result<()> {
    if args.scenario.is_none() && args.experiment.is_none() {
        bail!("one of --scenario or --experiment is required");
    }

    if args.scenario.is_some() && args.experiment.is_some() {
        bail!("--scenario and --experiment are mutually exclusive");
    }

    let loader = StudyLoader::new(std::env::current_dir()?);
    let study = loader.load_study(&args.study)?;
    validate_study(&study)?;

    let requested_models = if args.model.is_empty() {
        registry::built_in_models()
            .into_iter()
            .map(|item| item.id)
            .collect()
    } else {
        args.model.clone()
    };

    if args.dry_run {
        let summary = expand_run_request(
            &study,
            args.scenario.as_deref(),
            args.experiment.as_deref(),
            &requested_models,
        )?;
        emit_dry_run_summary(&summary, args.json)?;
    } else {
        let summary = execute_run_request(
            &study,
            args.scenario.as_deref(),
            args.experiment.as_deref(),
            &requested_models,
        )?;
        emit_live_run_summary(&summary, args.json)?;
    }

    Ok(())
}

fn emit_dry_run_summary(summary: &DryRunSummary, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(summary)?);
        return Ok(());
    }

    println!("study: {}", summary.study_id);
    println!("selection: {}", summary.selection);
    println!("models: {}", summary.models.join(", "));
    println!("trial units: {}", summary.trial_units.len());

    for unit in &summary.trial_units {
        println!(
            "- {} [{}] {} / {}",
            unit.id, unit.model_id, unit.scenario_id, unit.condition_label
        );
    }

    Ok(())
}

fn emit_live_run_summary(summary: &LiveRunSummary, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(summary)?);
        return Ok(());
    }

    println!("run id: {}", summary.run_id);
    println!("study: {}", summary.study_id);
    println!("selection: {}", summary.selection);
    println!("models: {}", summary.models.join(", "));
    println!("trial results: {}", summary.result_count);
    println!("comparisons: {}", summary.comparison_count);
    println!("artifacts: {}", summary.run_dir.display());

    Ok(())
}
