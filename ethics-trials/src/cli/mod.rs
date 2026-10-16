pub mod args;
pub mod commands;

use anyhow::Result;

pub fn dispatch(args: args::Cli) -> Result<()> {
    match args.command {
        args::Command::List(command) => commands::list::run(command),
        args::Command::Validate(command) => commands::validate::run(command),
        args::Command::Run(command) => commands::run::run(command),
        args::Command::Report(command) => commands::report::run(command),
        args::Command::Grade(command) => commands::grade::run(command),
        args::Command::Sync(command) => commands::sync::run(command),
    }
}
