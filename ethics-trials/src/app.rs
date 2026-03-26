use anyhow::Result;

pub fn run() -> Result<()> {
    let args = crate::cli::args::Cli::parse_args();
    crate::cli::dispatch(args)
}
