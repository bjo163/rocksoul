# Rocksoul

**One Core. Infinite Evolution.**

Rocksoul is an experimental, Rust-first intelligence runtime that starts as one native CLI executable and is designed to grow into a platform for building and operating many kinds of applications.

The first milestone is intentionally small:

- One Rust project and one executable: rocksoul.exe on Windows.
- CLI-first interaction through an interactive REPL.
- OpenAI-compatible model endpoint support, including local inference servers such as Ollama.
- A portable root directory: all configuration and runtime state stay beside rocksoul.exe.
- Local RID identity/progression profile with 99 unrated capability dimensions.
- A keyboard-first Ratatui Cognitive Cockpit.
- A deterministic offline Core Integrity evaluation suite.
- A bounded one-request-per-turn model-call budget.
- No desktop app, web app, server, plugin marketplace, autonomous tool execution, or multi-agent framework in the seed release.

Rocksoul is a foundation for agent research and self-evolving software. It is not AGI, and the initial release does not modify or replace its own executable.

## Quick start

### Requirements

- Rust stable (1.85 or newer).
- A running OpenAI-compatible chat-completions endpoint and an available model.

### Build

    git clone https://github.com/bjo163/rocksoul.git
    cd rocksoul
    cargo build --release
    .\target\release\rocksoul.exe --help

The Windows executable is written to target/release/rocksoul.exe.

### Configure with .env

All mutable state is rooted at the directory containing rocksoul.exe. Rocksoul never writes Rocksoul configuration, identity, memory, cache, workspace, logs, or evaluation data to AppData or the user profile. The .env file lives at the root; config.toml lives in config/.

1. Run initialization once:

    .\target\release\rocksoul.exe init

2. Edit target/release/.env:

    # Local Ollama example
    ROCKSOUL_BASE_URL=http://localhost:11434/v1
    ROCKSOUL_MODEL=your-installed-model
    ROCKSOUL_API_KEY=

    # Hosted OpenAI-compatible endpoint example
    # ROCKSOUL_BASE_URL=https://provider.example/v1
    # ROCKSOUL_MODEL=your-model-id
    # ROCKSOUL_API_KEY=your-api-key

3. Start Rocksoul:

    .\target\release\rocksoul.exe

For a downloaded executable, place .env beside rocksoul.exe, then run rocksoul.exe init if you need the template.

Precedence: environment variables already set in Windows/PowerShell take priority over values in .env. The .env file is never overwritten by init. Keep real API keys out of Git; .env is ignored by .gitignore.

The Rocksoul root must be writable for init and runtime data. The Windows installer grants write access only to mutable folders and .env, not to the executable itself.

### CLI

    rocksoul
    rocksoul init
    rocksoul status
    rocksoul doctor
    rocksoul tui
    rocksoul rid status
    rocksoul rid stats
    rocksoul rid history
    rocksoul eval list
    rocksoul eval run core
    rocksoul eval latest
    rocksoul --help

Inside the interactive CLI:

- :help — show available REPL commands.
- :status — show runtime and model configuration status.
- :rid — show instance age, level, XP and stat coverage.
- :clear — clear the current conversation.
- :exit — leave the REPL.

Chat history is session-only. Each model turn has a bounded request count, 16,000-token output ceiling (sent as `max_tokens`), and 120-second HTTP timeout. Provider-reported output usage is shown when available; providers that omit usage are marked unavailable. A successful model response is not proof that the user's task was completed. No XP is awarded for ordinary chat. Persistent memory and tool execution remain future milestones.

### RID and evaluation

`rocksoul init` creates a local RID profile. Its age is elapsed time since initialization; XP and levels are separate from capability scores. All 99 stat slots start unrated and require trusted, evidence-backed evaluation.

Run the offline deterministic suite with `rocksoul eval run core`. It checks kernel lifecycle, bounded retries, terminal-state immutability, verified-XP gating and replay behavior, RID initialization, catalogue integrity, and level progression. Reports are saved inside the Rocksoul root at `data/evaluations/core/latest.json`.

These checks measure bounded software invariants; they do not demonstrate AGI, consciousness, or general intelligence. External benchmark adapters are not yet integrated.

## Configuration

| Variable | Purpose | Default |
|---|---|---|
| ROCKSOUL_BASE_URL | OpenAI-compatible API base URL | http://localhost:11434/v1 |
| ROCKSOUL_MODEL | Model identifier | Required |
| ROCKSOUL_API_KEY | Optional bearer token | Unset |

The API base URL should include the version prefix when required by the provider, for example /v1. Rocksoul sends requests to /chat/completions.

## Design principles

1. Single core first. Keep one Cargo project and one executable until a real boundary justifies splitting it.
2. Model-independent. The core talks to an OpenAI-compatible API; it does not own or train a foundation model.
3. CLI-first. Other interfaces must be built on top of the same core later.
4. Capabilities are explicit. A model response is not proof that a command ran or a file changed.
5. Evolution is evaluated. Future self-modification must create isolated candidates, run tests and evaluations, preserve lineage, and support rollback.
6. Safe by default. No arbitrary shell execution, silent file mutation, or automatic promotion of modified core code in the seed release.
7. Portable source, native output. Rust remains the core implementation language; future tools may use other runtimes when justified.

## Repository map

    src/
      cli.rs          CLI argument definitions
      commands.rs     command handlers
      core.rs         cognitive turn, verification, and budget contracts
      eval.rs         offline capability evaluation harness
      lib.rs          application entry point and environment bootstrap
      main.rs         native executable entry point
      model.rs        bounded OpenAI-compatible chat client
      paths.rs        platform paths and executable-local .env
      repl.rs         bounded interactive turn loop
      rid.rs          RID identity, XP ledger, and 99 stat domain
      rid_catalogue.rs validated embedded 99-stat catalogue
      rid_store.rs    local RID persistence
      tui.rs          Ratatui Cognitive Cockpit
    docs/
      ARCHITECTURE.md
      ROADMAP.md
    SECURITY.md
    AGENTS.md
    .github/workflows/

## Roadmap

See docs/ROADMAP.md for staged development. The intended progression is:

CLI seed → cognition → memory → skills/tools → application builder → isolated evolution → multi-agent capabilities → additional interfaces.

## Development

    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test --all-targets
    cargo build --release

See AGENTS.md for repository rules and SECURITY.md for the security model.

## Project status

Seed runtime in active development. The CLI, RID, TUI, cognitive turn contracts, and offline core evaluation are implemented on the current integration branch; merge/release status is tracked by GitHub PRs and CI. Persistent memory, tool adapters, full plan/execute/verify task orchestration, external benchmark adapters, and controlled self-evolution are not implemented yet.
