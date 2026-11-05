use clap::{Parser, Subcommand};

use crate::memory::MemoryKind;

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
    /// Run and inspect offline capability evaluations.
    Eval {
        #[command(subcommand)]
        command: EvalCommand,
    },
    /// Create a user-approved text artifact in the portable workspace.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    /// Manage explicit, user-approved local memories.
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum EvalCommand {
    /// List available evaluation suites.
    List,
    /// Run the deterministic offline core integrity suite.
    Run {
        #[command(subcommand)]
        suite: EvalSuite,
    },
    /// Show the latest saved core integrity report.
    Latest,
    /// Show the latest saved memory retrieval evaluation report.
    MemoryLatest,
}

#[derive(Debug, Subcommand)]
pub enum EvalSuite {
    Core,
    MemoryRetrieval,
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

#[derive(Debug, Subcommand)]
pub enum WorkspaceCommand {
    /// Inspect the local append-only task trace.
    Trace,
    /// Create a UTF-8 .txt artifact after exact interactive confirmation.
    Write {
        filename: String,
        #[arg(long)]
        content: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    /// Add an explicitly user-approved memory record.
    Add {
        #[arg(value_enum)]
        kind: MemoryKind,
        #[arg(long)]
        content: String,
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Expire a working-memory entry after this many hours.
        #[arg(long)]
        expires_in_hours: Option<u64>,
    },
    /// List active memory records.
    List,
    /// Search active memory using deterministic lexical ranking.
    Search {
        query: String,
        #[arg(long, value_enum)]
        kind: Option<MemoryKind>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        created_after: Option<u64>,
        #[arg(long)]
        created_before: Option<u64>,
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },
    /// Permanently delete one memory record by ID.
    Delete { id: String },
    /// Permanently delete all memory records (requires --confirm).
    Clear {
        #[arg(long)]
        confirm: bool,
    },
    /// Preview expired records; pass --apply to permanently forget them.
    PruneExpired {
        #[arg(long)]
        apply: bool,
    },
    /// Export active memory to data/memory/exports/latest.json.
    Export,
}
