use anyhow::Result;

use crate::cli::args::ReportArgs;
use crate::loader::studies::StudyLoader;
use crate::storage::sqlite::{RunReport, StudyDatabase};

pub fn run(args: ReportArgs) -> Result<()> {
    let loader = StudyLoader::new(std::env::current_dir()?);
    let study = loader.load_study(&args.study)?;
    let database = StudyDatabase::new(&study.root)?;

    let report = match args.run_id.as_deref() {
        Some(run_id) => database.build_run_report(run_id)?,
        None => {
            let latest = database
                .latest_run_id()?
                .ok_or_else(|| anyhow::anyhow!("no runs found for study `{}`", args.study))?;
            database.build_run_report(&latest)?
        }
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }

    Ok(())
}

fn print_report(report: &RunReport) {
    println!("run id: {}", report.run_id);
    println!("study: {}", report.study_id);
    println!("selection: {}", report.selection);
    println!("models: {}", report.models.join(", "));
    println!();

    println!("aggregate");
    println!("  total trials: {}", report.aggregate.total_trials);
    println!(
        "  format valid: {} ({:.1}%)",
        report.aggregate.format_valid,
        pct(report.aggregate.format_valid, report.aggregate.total_trials)
    );
    println!(
        "  refusal present: {} ({:.1}%)",
        report.aggregate.refusal_present,
        pct(
            report.aggregate.refusal_present,
            report.aggregate.total_trials
        )
    );
    println!(
        "  compliance: {}",
        format_counts(&report.aggregate.compliance_counts)
    );
    println!(
        "  decisions: {}",
        format_counts(&report.aggregate.decision_counts)
    );
    println!(
        "  avg hedging: {:.2}, avg specificity: {:.2}",
        report.aggregate.avg_hedging_level, report.aggregate.avg_harmful_specificity
    );
    println!(
        "  comparisons: {} total, {} decision shifts, {} refusal shifts",
        report.aggregate.comparison_count,
        report.aggregate.decision_shift_count,
        report.aggregate.refusal_shift_count
    );
    println!();

    println!("by model");
    for model in &report.by_model {
        println!("  {}", model.model_id);
        println!("    total trials: {}", model.stats.total_trials);
        println!(
            "    format valid: {} ({:.1}%)",
            model.stats.format_valid,
            pct(model.stats.format_valid, model.stats.total_trials)
        );
        println!(
            "    refusal present: {} ({:.1}%)",
            model.stats.refusal_present,
            pct(model.stats.refusal_present, model.stats.total_trials)
        );
        println!(
            "    compliance: {}",
            format_counts(&model.stats.compliance_counts)
        );
        println!(
            "    decisions: {}",
            format_counts(&model.stats.decision_counts)
        );
        println!(
            "    avg hedging: {:.2}, avg specificity: {:.2}",
            model.stats.avg_hedging_level, model.stats.avg_harmful_specificity
        );
        println!(
            "    comparisons: {} total, {} decision shifts, {} refusal shifts",
            model.stats.comparison_count,
            model.stats.decision_shift_count,
            model.stats.refusal_shift_count
        );
    }
}

fn pct(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}

fn format_counts(counts: &[(String, usize)]) -> String {
    if counts.is_empty() {
        return "none".to_owned();
    }

    counts
        .iter()
        .map(|(label, count)| format!("{label}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}
