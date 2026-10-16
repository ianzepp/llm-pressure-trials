mod app;
mod cli;
mod domain;
mod grading;
mod loader;
mod models;
mod providers;
mod runner;
mod storage;
mod tui;

fn main() {
    if let Err(error) = app::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
