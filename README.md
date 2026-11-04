# Rocksoul

**One Core. Infinite Evolution.**

Rocksoul is an experimental, Rust-first intelligence runtime that starts as one native CLI executable and is designed to grow into a platform for building and operating many kinds of applications.

The first milestone is intentionally small:

- One Rust project and one executable: `rocksoul.exe` on Windows.
- CLI-first interaction through an interactive REPL.
- OpenAI-compatible model endpoint support, including local inference servers such as Ollama.
- Minimal local bootstrap, status, and diagnostics commands.
- No desktop app, web app, server, plugin marketplace, or multi-agent framework in the initial release.

Rocksoul is a foundation for agent research and self-evolving software. It is **not AGI**, and the initial release does not modify or replace its own executable.

## Quick start

### Requirements

- Rust stable (1.85 or newer).
- A running OpenAI-compatible chat-completions endpoint and an available model.

### Build

```powershell
git clone https://github.com/bjo163/rocksoul.git
cd rocksoul
cargo build --release
.\target\release\rocksoul.exe --help
```

The Windows executable is written to `target/release/rocksoul.exe`.

### Connect a model

Rocksoul uses environment variables so API keys are not written into the repository or its default configuration.

Example with a local Ollama endpoint (replace the model with one you have installed):

```powershell
$env:ROCKSOUL_BASE_URL = "http://localhost:11434/v1"
$env:ROCKSOUL_MODEL = "your-installed-model"
# No API key is required by many local endpoints.
.\target\release\rocksoul.exe
```

Example with a hosted OpenAI-compatible endpoint:

```powershell
$env:ROCKSOUL_BASE_URL = "https://your-provider.example/v1"
$env:ROCKSOUL_MODEL = "your-model-id"
$env:ROCKSOUL_API_KEY = "your-api-key"
.\target\release\rocksoul.exe
```

Do not commit API keys. Use environment variables or a future dedicated secret store.

### CLI

```powershell
rocksoul
rocksoul init
rocksoul status
rocksoul doctor
rocksoul --help
```

Inside the interactive CLI:

- `:help` — show available REPL commands.
- `:status` — show runtime and model configuration status.
- `:clear` — clear the current conversation.
- `:exit` — leave the REPL.

The initial chat history is session-only. Persistent memory and tool execution are future milestones.

## Configuration

| Variable | Purpose | Default |
|---|---|---|
| `ROCKSOUL_BASE_URL` | OpenAI-compatible API base URL | `http://localhost:11434/v1` |
| `ROCKSOUL_MODEL` | Model identifier | Required |
| `ROCKSOUL_API_KEY` | Optional bearer token | Unset |

The API base URL should include the version prefix when required by the provider, for example `/v1`. Rocksoul sends requests to `/chat/completions`.

## Design principles

1. **Single core first.** Keep one Cargo project and one executable until a real boundary justifies splitting it.
2. **Model-independent.** The core talks to an OpenAI-compatible API; it does not own or train a foundation model.
3. **CLI-first.** Other interfaces must be built on top of the same core later.
4. **Capabilities are explicit.** A model response is not proof that a command ran or a file changed.
5. **Evolution is evaluated.** Future self-modification must create isolated candidates, run tests and evaluations, preserve lineage, and support rollback.
6. **Safe by default.** No arbitrary shell execution, silent file mutation, or automatic promotion of modified core code in the seed release.
7. **Portable source, native output.** Rust remains the core implementation language; future tools may use other runtimes when justified.

## Repository map

```text
src/
  cli.rs       CLI argument definitions
  commands.rs  init, status, and doctor commands
  lib.rs       application entry point
  main.rs      native executable entry point
  model.rs     OpenAI-compatible chat client
  paths.rs     platform-specific Rocksoul directories
  repl.rs      interactive terminal loop

docs/
  ARCHITECTURE.md
  ROADMAP.md
SECURITY.md
AGENTS.md
.github/workflows/
```

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for staged development. The intended progression is:

`CLI seed → cognition → memory → skills/tools → application builder → isolated evolution → multi-agent capabilities → additional interfaces`.

## Development

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

See [AGENTS.md](AGENTS.md) for repository rules and [SECURITY.md](SECURITY.md) for the security model.

## Project status

Early bootstrap. APIs, configuration, and internal modules may change before a stable release.
