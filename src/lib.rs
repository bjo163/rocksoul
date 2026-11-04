mod cli;
mod commands;
mod model;
mod paths;
mod repl;

use anyhow::Result;
use clap::Parser;

pub async fn run() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(command) => commands::run(command),
        None => repl::run().await,
    }
}
