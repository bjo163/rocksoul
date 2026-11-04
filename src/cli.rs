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
}
