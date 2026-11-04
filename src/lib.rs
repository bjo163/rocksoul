mod cli;
mod commands;
pub mod core;
mod eval;
mod memory;
mod model;
mod paths;
mod repl;
pub mod rid;
pub mod rid_catalogue;
pub mod task_trace;
mod rid_store;
mod tui;
pub mod workspace_artifact;

use anyhow::Result;
use clap::Parser;

/// Load the dotenv file beside the current executable before starting async workers.
/// Existing process environment variables intentionally take precedence.
pub fn load_environment() -> Result<bool> {
    paths::load_executable_dotenv()
}

pub async fn run() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(command) => commands::run(command),
        None => repl::run().await,
    }
}
