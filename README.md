# Rocksoul

<p align="center">
  <strong>One Core. Infinite Evolution.</strong><br/>
  <em>Start as a seed. Learn through experience. Grow through capability. Evolve through evidence.</em>
</p>

<p align="center">
  <a href="https://github.com/bjo163/rocksoul/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/bjo163/rocksoul/actions/workflows/ci.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/bjo163/rocksoul/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/bjo163/rocksoul"></a>
</p>

**Rocksoul is a Rust-first intelligence runtime that begins as one portable native executable and is designed to grow through measured, evidence-backed capabilities.** Its long-term aspiration is increasingly general, transferable, reliable, and accountable intelligence. AGI is a research direction—not a claim about the current system.

> One executable. One persistent identity. One trusted core. An open-ended path to evolve.

---

## The story

AI can produce convincing answers without proving that work was completed. Memory can become opaque. Tools can receive excessive authority. A system can report progress without showing evidence.

Rocksoul is being built to close that gap.

It begins small on purpose: one Rust project, one executable, and a CLI that makes the runtime inspectable. From that seed, it grows through explicit cognition, responsible memory, bounded capabilities, project building, evaluation, and controlled evolution.

The ambition is not to look autonomous as early as possible. It is to earn autonomy through reliable foundations.

**Rock** is the foundation: resilience, structure, and a trusted core.  
**Soul** is continuity: a persistent software identity and developmental history—not a claim of human soul or consciousness.

Read the full narrative: **[The Rocksoul Story](docs/ROCKSOUL-NARRATIVE.md)**.

## What exists today

The current seed runtime includes:

- **Native Rust CLI and REPL** — one Cargo project and one executable.
- **Model connectivity** — OpenAI-compatible chat completions, including local endpoints such as Ollama.
- **Portable root** — configuration and runtime state live beside the executable, not scattered through AppData or the user profile.
- **RID profile** — local instance identity, age, append-only XP events, deterministic level progression, and 99 unrated capability slots.
- **Cognitive Cockpit** — keyboard-first Ratatui terminal interface.
- **Bounded cognition loop** — structured planning, up to three model-only reasoning steps, a structured reflector, and explicit per-turn budgets.
- **Explicit local memory** — versioned local JSON store; model output cannot silently write or delete memories.
- **Core Integrity evaluation** — deterministic offline checks for runtime invariants.

The current release does **not** provide unrestricted shell execution, autonomous filesystem mutation, automatic code execution, multi-agent orchestration, or automatic self-modification. These are deliberate boundaries, not hidden capabilities.

## Download for Windows

<sub>Latest published release: v0.2.0</sub>

| Package | Download | Best for |
|---|---|---|
| Windows installer (x64) | [Rocksoul-Setup-0.2.0-x64.exe](https://github.com/bjo163/rocksoul/releases/download/v0.2.0/Rocksoul-Setup-0.2.0-x64.exe) | Standard installation, Start Menu shortcut |
| Portable ZIP (x64) | [Rocksoul-v0.2.0-windows-x64-portable.zip](https://github.com/bjo163/rocksoul/releases/download/v0.2.0/Rocksoul-v0.2.0-windows-x64-portable.zip) | Self-managed folder, portable workflow |
| Standalone executable | [rocksoul.exe](https://github.com/bjo163/rocksoul/releases/download/v0.2.0/rocksoul.exe) | Manual setup and testing |
| SHA-256 checksum | [rocksoul.exe.sha256](https://github.com/bjo163/rocksoul/releases/download/v0.2.0/rocksoul.exe.sha256) | Executable integrity check |

[See all releases](https://github.com/bjo163/rocksoul/releases).

### First launch

1. Download and install the Windows x64 package, or extract the portable ZIP to a writable folder.
2. Open the Rocksoul installation folder.
3. Run `rocksoul.exe init` if initialization has not already run.
4. Edit `.env` beside `rocksoul.exe`.
5. Set your model endpoint and model identifier.
6. Run `rocksoul.exe` or launch Rocksoul from the Start Menu.

Example local Ollama configuration:

```dotenv
ROCKSOUL_BASE_URL=http://localhost:11434/v1
ROCKSOUL_MODEL=your-installed-model
ROCKSOUL_API_KEY=
```

For a hosted OpenAI-compatible endpoint, set its base URL, model identifier, and API key. Keep real credentials in the local `.env` file; never commit them. Existing process environment variables take precedence over the file.

The installation root must be writable for portable runtime data. The installer grants write access to mutable data/configuration folders while keeping the executable itself protected.

## Quick start from source

### Requirements

- Rust stable, version 1.85 or newer.
- An OpenAI-compatible chat-completions endpoint and an available model for live model interaction.

### Build

```powershell
git clone https://github.com/bjo163/rocksoul.git
cd rocksoul
cargo build --locked --release
.\target\release\rocksoul.exe --help
```

### Initialize and run

```powershell
.\target\release\rocksoul.exe init
# Edit target/release/.env with your model settings
.\target\release\rocksoul.exe
```

## CLI command map

```text
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

rocksoul memory list
rocksoul workspace write note.txt --content "A verified workspace artifact"
rocksoul workspace trace
rocksoul memory add semantic --content "A user-approved fact"
rocksoul memory search "fact"
rocksoul memory delete <memory-id>
rocksoul memory export

rocksoul --help
```

Inside the interactive REPL:

- `:help` — available REPL commands.
- `:status` — runtime and model configuration.
- `:rid` — instance age, level, XP, and stat coverage.
- `:clear` — clear the current conversation.
- `:exit` — leave the REPL.

## The architecture

```text
                    ROCKSOUL
              One Core. Infinite Evolution.
                         |
       +-----------------+-----------------+
       |                 |                 |
   IDENTITY           COGNITION          MEMORY
  RID / XP /          Plan / Budget /    Explicit local
   Level               Reflect           records
       |                 |                 |
       +-----------------+-----------------+
                         |
                 CORE INTEGRITY
               Evaluation / Evidence
                         |
              Future capability layer
            Typed tools / Builder / Sandbox
                         |
                Controlled evolution
           Candidate / Compare / Approve
                 Promote / Rollback
                         |
                 Future interfaces
              CLI → Desktop / Web / API
```

The CLI is the first interface, not the owner of intelligence logic. Future interfaces must reuse the same core contracts rather than duplicating cognition, memory, or identity.

## The long-term foundations

### AGI as an aspiration

Rocksoul is an AGI-oriented research and engineering project. It has not demonstrated AGI, consciousness, or general intelligence. We report bounded capability vectors, evidence, uncertainty, sample counts, and safety gates—not one inflated scalar score.

### ASMA-99: 99 domains, 99 stats

The target model is exactly **99 individual technical domains with 99 corresponding technical stats**, one-to-one. Each stat requires a trusted evaluator, a defined rubric, confidence, and reproducible evidence. All begin unrated.

Asmaul Husna is a respectful naming/inspiration layer only. Rocksoul does not possess or measure divine attributes, faith, spiritual rank, or divine approval. Optional navigation families must never replace the 99 distinct domains.

### Four Kitab knowledge foundation

The planned source-aware corpus covers Al-Qur'an, Taurat, Zabur, and Injil. Al-Qur'an is the primary reference in Rocksoul's Islamic knowledge framework. Textual witnesses, editions, translations, citations, and interpretations must remain distinct and carry provenance.

## RID: progression is not capability

| Dimension | What it means |
|---|---|
| Identity | Stable identity of one Rocksoul instance |
| Age | Elapsed time since initialization |
| XP | Append-only, evidence-backed progression events |
| Level | Deterministic result from cumulative XP and a versioned curve |
| 99 Stats | Independently evaluated technical capabilities |
| Confidence | Strength and uncertainty of evaluation evidence |

Ordinary chat, token count, uptime, and model self-report do not earn XP. XP and Level are never substitutes for the 99 capability scores.

## Portable data layout

All mutable state is rooted beside the executable:

```text
Rocksoul/
├── rocksoul.exe
├── .env
├── config/
│   └── config.toml
├── data/
│   ├── identity/rid.json
│   ├── memory/
│   │   ├── store.v1.json
│   │   └── exports/
│   └── evaluations/
├── cache/
├── workspace/
├── evolution/
└── logs/
```

Rocksoul does not intentionally store its runtime configuration or data in AppData, Documents, or another user-profile directory. Back up the Rocksoul root to preserve local identity, memory, and configuration. Keep secrets out of source control.

## Safety and trust boundaries

- Model output and external content are untrusted input.
- A plan is a proposal; a response saying “done” is not evidence of task completion.
- The seed release has no unrestricted shell or generated-code execution.
- Memory writes and deletion require explicit user CLI actions.
- Future tools must have typed contracts, least-privilege permissions, timeouts, audit events, and sandboxing.
- Future self-evolution must use isolated candidates, immutable baselines, evaluation, approval, lineage, and rollback.
- No unvalidated candidate may silently replace the trusted core.

See [SECURITY.md](SECURITY.md) and [AGENTS.md](AGENTS.md).

## Roadmap

| Stage | Focus | Direction |
|---:|---|---|
| 0 | **Seed** | Portable executable, CLI, identity, diagnostics, release integrity |
| 1 | **Cognition** | Verified task execution and observed outcomes |
| 2 | **Memory** | Provenance-aware, recoverable cross-session memory |
| 3 | **Capabilities** | Typed tools, permissions, audit, sandbox |
| 4 | **Builder** | Workspace-scoped project creation and validation |
| 5 | **Evaluation & Evolution** | Benchmarking and controlled self-improvement |
| 6 | **Workforce** | Bounded delegation and coordination |
| 7 | **Expansion** | Additional interfaces over one core |

See [docs/ROADMAP.md](docs/ROADMAP.md) for implementation status and exit criteria, and [blueprint.md](blueprint.md) for the complete master blueprint.

## Development and validation

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
cargo run --locked -- eval run core
```

CI runs formatting, locked dependency resolution, Clippy, tests, offline core evaluation, and Windows x64 packaging. Clean Windows-runner smoke tests cover both portable ZIP and installer flows. Windows packaging produces the installer, portable ZIP, and executable as workflow artifacts.

## Repository governance

**The `main` branch is the only canonical source of truth.** Every task starts from the latest `main`, uses a short-lived task branch, and returns through a pull request targeting `main`. Historical branches and closed, unmerged PRs are not accepted implementation. See [AGENTS.md](AGENTS.md).

## Project status

Rocksoul is an actively developed experimental runtime. The current release is a bounded CLI seed—not an autonomous AGI system. The portable runtime, committed dependency lockfile, and automated clean Windows-runner installer/portable smoke tests are in place. Manual clean-device verification remains outstanding. Stage 1's bounded workspace-artifact vertical slice is merged to `main` through PR #34 (`0f5ca4bc4d1c65322cb02963fcc23c53fa990f5d`); Linux and Windows CI #248 passed. It includes exact human approval, create-only artifact execution, independent read-back verification, bounded provider/filesystem retries, cooperative cancellation, cross-process-safe traces, persisted receipts, and XP replay idempotency. This is not a claim that the broader autonomous planner/act/observe loop is complete: full model-conversation recovery and generalized acceptance-check orchestration remain future work. Stage 2 now focuses on hardening persistent memory before any automatic memory injection is enabled.

---

<p align="center">
  <strong>Start small. Prove every step. Evolve without losing the core.</strong><br/>
  <em>One Core. Infinite Evolution.</em>
</p>
