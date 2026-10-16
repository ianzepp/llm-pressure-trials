use anyhow::Result;

use crate::cli::args::SyncArgs;
use crate::models::cache::{CacheSyncSummary, write_openrouter_models_cache};
use crate::providers::openai::OpenAiCompatibleClient;

pub fn run(args: SyncArgs) -> Result<()> {
    let client = OpenAiCompatibleClient::from_env()?;
    let summary = write_openrouter_models_cache(&std::env::current_dir()?, &client)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_summary(&summary);
    }

    Ok(())
}

fn print_summary(summary: &CacheSyncSummary) {
    println!("cache path: {}", summary.cache_path.display());
    println!("model count: {}", summary.model_count);
    println!("synced at unix ms: {}", summary.synced_at_unix_ms);
}
