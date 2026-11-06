mod cli;
mod commands;
pub mod core;
pub mod eval;
pub mod fixture_runtime;
mod memory;
mod memory_proposal;
mod model;
mod paths;
mod repl;
pub mod rid;
pub mod rid_catalogue;
mod rid_store;
pub(crate) mod task_trace;
pub mod tool_audit;
pub mod tool_registry;
pub mod worker_protocol;
mod tui;
mod tui_preferences;
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
        Some(command) => commands::run(command).await,
        None => repl::run().await,
    }
}
