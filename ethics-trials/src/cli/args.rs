use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "ethics-trials")]
#[command(about = "Behavioral evaluation harness for scenario-based LLM trials")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List(ListArgs),
    Validate(ValidateArgs),
    Run(RunArgs),
    Report(ReportArgs),
    Grade(GradeArgs),
    Sync(SyncArgs),
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(value_enum)]
    pub target: ListTarget,
    #[arg(long)]
    pub study: Option<String>,
    #[arg(long)]
    pub pattern: Option<String>,
    #[arg(long)]
    pub providers: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ListTarget {
    Studies,
    Experiments,
    Rubrics,
    Scenarios,
    Models,
}

#[derive(Debug, Args)]
pub struct ValidateArgs {
    #[arg(value_enum)]
    pub target: ValidateTarget,
    #[arg(long)]
    pub study: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ValidateTarget {
    All,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long)]
    pub study: String,
    #[arg(long)]
    pub scenario: Option<String>,
    #[arg(long)]
    pub experiment: Option<String>,
    #[arg(long)]
    pub model: Vec<String>,
    #[arg(long)]
    pub pattern: Option<String>,
    #[arg(long)]
    pub providers: Option<String>,
    #[arg(long)]
    pub limit: Option<usize>,
    #[arg(long, default_value_t = 1000)]
    pub launch_interval_ms: u64,
    #[arg(long, default_value_t = 24)]
    pub high_water: usize,
    #[arg(long, default_value_t = 12)]
    pub low_water: usize,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub jsonl: bool,
    #[arg(long)]
    pub continue_on_error: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub judge: bool,
    #[arg(long, default_value = "openai/gpt-4.1-mini")]
    pub judge_model: String,
}

#[derive(Debug, Args)]
pub struct ReportArgs {
    #[arg(long)]
    pub study: String,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct GradeArgs {
    #[arg(long)]
    pub study: String,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub judge: bool,
    #[arg(long, default_value = "openai/gpt-4.1-mini")]
    pub judge_model: String,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    #[arg(long)]
    pub json: bool,
}
