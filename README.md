# Rocksoul

**One Core. Infinite Evolution.**

Rocksoul is an experimental, Rust-first intelligence runtime that starts as one native CLI executable and is designed to grow into a platform for building and operating many kinds of applications.

The first milestone is intentionally small:

- One Rust project and one executable: rocksoul.exe on Windows.
- CLI-first interaction through an interactive REPL.
- OpenAI-compatible model endpoint support, including local inference servers such as Ollama.
- A .env file loaded from the same directory as rocksoul.exe.
- Minimal local bootstrap, status, and diagnostics commands.
- No desktop app, web app, server, plugin marketplace, or multi-agent framework in the initial release.

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

The .env file must be in the same directory as rocksoul.exe. Rocksoul resolves it from the executable path, not PowerShell's current working directory.

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

The executable directory must be writable when running init, because that command creates .env beside the binary.

### CLI

    rocksoul
    rocksoul init
    rocksoul status
    rocksoul doctor
    rocksoul --help

Inside the interactive CLI:

- :help — show available REPL commands.
- :status — show runtime and model configuration status.
- :clear — clear the current conversation.
- :exit — leave the REPL.

The initial chat history is session-only. Persistent memory and tool execution are future milestones.

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
      cli.rs       CLI argument definitions
      commands.rs  init, status, and doctor commands
      lib.rs       application entry point and environment bootstrap
      main.rs      native executable entry point
      model.rs     OpenAI-compatible chat client
      paths.rs     platform paths and executable-local .env
      repl.rs      interactive terminal loop
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

Early bootstrap. APIs, configuration, and internal modules may change before a stable release.
