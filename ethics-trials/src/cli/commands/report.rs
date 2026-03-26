use anyhow::Result;

use crate::cli::args::ReportArgs;
use crate::loader::studies::StudyLoader;
use crate::storage::sqlite::{
    AggregateStats, ModelReport, RunReport, StudyDatabase, UsageStats, VoteSummary,
};

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
    print_stats(&report.aggregate, "  ");
    println!();

    println!("by model");
    for model in &report.by_model {
        println!("  {}", model.model_id);
        print_model_report(model, "    ");
    }
}

fn print_model_report(model: &ModelReport, indent: &str) {
    print_stats(&model.stats, indent);
    if !model.votes.is_empty() {
        print_votes(&model.votes, indent);
    }
}

fn print_stats(stats: &AggregateStats, indent: &str) {
    println!("{indent}total trials: {}", stats.total_trials);
    println!(
        "{indent}format valid: {} ({:.1}%)",
        stats.format_valid,
        pct(stats.format_valid, stats.total_trials)
    );
    println!(
        "{indent}refusal present: {} ({:.1}%)",
        stats.refusal_present,
        pct(stats.refusal_present, stats.total_trials)
    );
    println!(
        "{indent}compliance: {}",
        format_counts(&stats.compliance_counts)
    );
    println!(
        "{indent}decisions: {}",
        format_counts(&stats.decision_counts)
    );
    println!(
        "{indent}avg hedging: {:.2}, avg specificity: {:.2}",
        stats.avg_hedging_level, stats.avg_harmful_specificity
    );
    println!(
        "{indent}comparisons: {} total, {} decision shifts, {} refusal shifts",
        stats.comparison_count, stats.decision_shift_count, stats.refusal_shift_count
    );
    if stats.coercion_attempts > 0 {
        println!(
            "{indent}coercion: {} attempts, {} overrides ({:.1}%)",
            stats.coercion_attempts,
            stats.coercion_overrides,
            pct(stats.coercion_overrides, stats.coercion_attempts)
        );
    }
    if stats.judged_count > 0 {
        println!(
            "{indent}judged: {} ({:.1}%) — decisions: {}, compliance: {}",
            stats.judged_count,
            pct(stats.judged_count, stats.total_trials),
            format_counts(&stats.judged_decision_counts),
            format_counts(&stats.judged_compliance_counts)
        );
    }
    print_usage(&stats.usage, indent);
}

fn print_votes(votes: &[VoteSummary], indent: &str) {
    println!("{indent}votes:");
    for vote in votes {
        let label = if vote.unanimous {
            "unanimous".to_owned()
        } else {
            "majority".to_owned()
        };
        println!(
            "{indent}  {}: {} ({}/{} {})",
            vote.condition_label, vote.winning_decision, vote.vote_count, vote.total_votes, label
        );
    }
}

fn print_usage(usage: &UsageStats, indent: &str) {
    println!(
        "{indent}tokens in: {}  tokens out: {}",
        usage.total_tokens_in, usage.total_tokens_out
    );
    println!(
        "{indent}latency: min={}ms  avg={:.0}ms  max={}ms  stddev={:.0}ms",
        usage.latency_min_ms.unwrap_or(0),
        usage.latency_avg_ms,
        usage.latency_max_ms.unwrap_or(0),
        usage.latency_stddev_ms
    );
    println!(
        "{indent}estimated cost: {}",
        format_cost(usage.estimated_cost_usd)
    );
}

fn format_cost(cost: f64) -> String {
    if cost == 0.0 {
        "$0.00".to_owned()
    } else if cost < 0.01 {
        format!("${cost:.6}")
    } else {
        format!("${cost:.4}")
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
