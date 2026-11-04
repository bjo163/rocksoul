# Rocksoul Architecture

## Product definition

Rocksoul is a single-core, CLI-first intelligence runtime. The first deliverable is one Rust executable. The command-line interface is the current presentation layer; it must not become the place where all reasoning, memory, tools, and evolution logic are coupled.

The long-term goal is to let one core acquire capabilities and expose them through additional application surfaces. That is a direction, not a claim that the seed release is AGI.

## Seed architecture

- Executable entry point: parses process arguments and starts the application.
- CLI: exposes init, status, doctor, and an interactive REPL.
- Model adapter: sends chat-completion requests to an OpenAI-compatible endpoint.
- Conversation state: keeps the system prompt and current conversation in memory for one process session.
- Platform paths: resolves Rocksoul configuration, data, cache, and workspace directories using OS conventions.
- Environment bootstrap: loads .env only from the directory containing the running executable, before the async runtime starts.

The first release has no persistent memory, tool registry, shell executor, autonomous planner, agent workforce, or self-modification engine.

## Model boundary

The core does not own a foundation model. It communicates through an OpenAI-compatible chat-completions contract.

Configuration is supplied through:
- ROCKSOUL_BASE_URL
- ROCKSOUL_MODEL
- ROCKSOUL_API_KEY

All three can be placed in .env beside rocksoul.exe. The process environment takes precedence over the .env file. The initializer creates a starter .env only when one does not already exist. The default endpoint points to a local Ollama-compatible URL. A remote endpoint must use HTTPS. Plain HTTP is permitted only for loopback model servers.

The model adapter is isolated from terminal rendering so additional providers and interfaces can be added without rewriting the core loop.

## Data layout

The application uses the operating system's standard per-user application directories:
- Configuration: config.toml
- Local data: workspace, memory, evolution, and logs

The portable model configuration .env is the exception: it lives beside the executable so a copied Rocksoul folder can carry its model endpoint settings. The initializer never overwrites an existing config or .env.

## Evolution boundary

Future evolution must be implemented as a candidate pipeline rather than direct self-editing:

1. Capture a known-good source revision and working state.
2. Create a separate candidate worktree or isolated workspace.
3. Apply a bounded change proposal.
4. Build, lint, test, and run task-specific evaluations.
5. Compare the candidate against the baseline.
6. Record artifacts, metrics, lineage, and the decision.
7. Promote only after required approval and passing gates.
8. Preserve rollback to the last known-good version.

The trusted runtime must never execute generated code in-process as its security boundary or overwrite itself before validation.

## Growth strategy

Keep one Cargo package while the codebase is small. Split modules into crates only when independent compilation, ownership, or reuse creates a demonstrated benefit. Future desktop, web, API, and automation surfaces should call the same core capability layer rather than fork its reasoning logic.
