use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rocksoul",
    version,
    about = "A Rust-first intelligence runtime, starting with one CLI executable",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create Rocksoul's local directories and initial configuration.
    Init,
    /// Display runtime and configuration status.
    Status,
    /// Check the local setup and model configuration.
    Doctor,
    /// Open the interactive terminal cockpit.
    Tui,
    /// Inspect the local Ruh Information profile.
    Rid {
        #[command(subcommand)]
        command: RidCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum RidCommand {
    /// Display instance age, level, XP, and stat coverage.
    Status,
    /// List all 99 capability stat slots and their measurement state.
    Stats,
    /// Display the auditable XP progression history.
    History,
}
