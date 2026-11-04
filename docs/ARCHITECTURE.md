# Rocksoul Architecture

**Status:** Describes the implementation on `main` as of 2026-11-04. Future capabilities are explicitly marked as planned.

## Product definition

Rocksoul is a single-core, CLI-first Rust intelligence runtime. The current product is one Cargo package and one native executable. The CLI is the primary interaction and automation surface; the optional Ratatui Cognitive Cockpit is a terminal presentation over the same executable. Neither interface owns a separate intelligence runtime.

The long-term aspiration is increasingly general, transferable, reliable, and accountable intelligence. AGI is a research direction, not a demonstrated capability.

## Current runtime map

| Area | Current responsibility | Implementation boundary |
|---|---|---|
| Process/bootstrap | Load root-local `.env`, initialize Tokio, dispatch the application | `main.rs`, library entry point |
| CLI | Parse commands and route init/status/doctor/TUI/RID/eval/memory operations | `cli.rs`, `commands.rs` |
| Model adapter | OpenAI-compatible chat-completions requests | `model.rs` |
| Interactive REPL | Session-local conversation, bounded planner, model-only reasoning steps, reflector | `repl.rs`, `core.rs` |
| Turn contracts | Lifecycle, retry limits, cancellation token, budgets, evidence-verification and XP gate | `core.rs` |
| Portable paths | Resolve all mutable runtime paths beside the executable | `paths.rs` |
| Workspace artifact capability | Create-only, approved UTF-8 `.txt` artifacts with read-back verification; CLI only | `workspace_artifact.rs`, `commands.rs` |
| RID | Instance identity, age, XP ledger, deterministic level curve, 99 stat slots | `rid.rs`, `rid_store.rs`, `rid_catalogue.rs` |
| Memory | Explicit user-approved local records, schema v1, search/export/delete and backup recovery | `memory.rs`, `commands.rs` |
| Evaluation | Offline deterministic Core Integrity checks and latest JSON report | `eval.rs` |
| Cognitive Cockpit | Opt-in Ratatui terminal inspection/navigation | `tui.rs` |

This table describes implemented module responsibilities, not independent processes or packages.

## Runtime entry points

- `rocksoul init`: creates the portable directory layout, starter `config/config.toml`, root-local `.env`, RID profile, and empty memory store. Existing config, environment, identity, and memory are preserved.
- `rocksoul status` and `rocksoul doctor`: report local runtime, model configuration, RID, and memory diagnostics.
- `rocksoul rid ...`: inspect identity, progression, and capability-stat data.
- `rocksoul eval ...`: list/run Core Integrity and inspect its latest report.
- `rocksoul memory ...`: explicitly list, add, search, export, delete, or clear local records.
- `rocksoul workspace write <filename> --content <text>`: create one flat `.txt` artifact after typing the exact `WRITE <filename>` approval phrase; the service reopens and compares the published bytes, refuses overwrites, and awards XP only through the existing verifier.
- `rocksoul workspace trace`: inspect content-free JSONL lifecycle records stored under the portable `logs/` directory.
- `rocksoul tui`: open the optional full-screen terminal cockpit; it requires an interactive terminal.
- `rocksoul`: start the interactive REPL. Conversation history is held in process memory for that session.

## Model and cognition boundary

The model adapter communicates with an OpenAI-compatible chat-completions endpoint using `ROCKSOUL_BASE_URL`, `ROCKSOUL_MODEL`, and `ROCKSOUL_API_KEY`. Values may be provided by the process environment or by `.env` beside the executable; process environment values take precedence. Remote endpoints must use HTTPS; plain HTTP is reserved for loopback model servers.

The REPL cognition path currently:
1. Creates a bounded turn request.
2. Requests and validates a structured plan with one to three steps and required acceptance checks.
3. Executes each step as another model call, not as an external action.
4. Requests a structured model reflection over the plan and generated outputs.
5. Completes or fails the in-memory turn based on the structured reflection.

The maximum is five model calls, 16,000 aggregate output tokens, and 120 seconds per turn. The reflector is not an independent verifier. No shell, arbitrary function tools, plugin execution, or external side-effect executor is exposed in the seed runtime. Plan acceptance checks are not yet connected to independent observed effects; model-only turn completion does not award XP.

The core also contains a trusted evidence-verification contract that can award XP only after required evidence checks pass. This contract is covered by deterministic evaluation, but the interactive planner does not yet produce independently observed evidence for its own checks.

## Portable data model

All mutable runtime paths are rooted in the directory containing the running executable. Rocksoul does not intentionally use AppData, Documents, or another user-profile location.

```text
Rocksoul/
├── rocksoul[.exe]
├── .env
├── config/
│   └── config.toml
├── data/
│   ├── identity/
│   │   └── rid.json
│   ├── memory/
│   │   ├── store.v1.json
│   │   └── exports/
│   └── evaluations/
│       └── core/
│           └── latest.json
├── cache/
├── workspace/
├── evolution/
└── logs/
```

The exact directories are initialized by `rocksoul init`. Keep credentials in the local `.env`; never commit secrets. The installation root must be writable for the runtime data it contains.

### Memory implementation status

The current memory store is a local JSON schema v1 with typed record kinds (episodic, semantic, procedural, working, derived), provenance/source fields, explicit user approval, timestamps, optional confidence, and optional expiry. The CLI supports list, add, text/tag search, export, delete, and confirmed clear. Writes use a temporary-file/backup replacement strategy and loading can recover a backup when the primary file is absent.

Not implemented yet: schema migration, inter-process locking, retention/forgetting workflows beyond explicit deletion and working-record expiry, ranked retrieval, automatic conversation ingestion, and automatic memory injection into model context. Model output cannot silently write or delete memory.

## RID and ASMA-99 boundary

RID identity, age, XP, level, and capability stats are separate data dimensions. XP events require evidence references and unique IDs; level is derived from a versioned deterministic curve. The 99 stat slots start unrated.

The embedded catalogue currently contains 99 unique technical stat entries. Its technical entries are presently grouped under nine shared domain labels (eleven stats per group). This is a transitional catalogue shape, not the final ASMA-99 target. The target is 99 individually identified technical domains, each mapped one-to-one to one stat. The nine existing labels may remain as optional navigation families only after a versioned migration that preserves slot IDs and evaluation history.

Asmaul Husna is a respectful naming/inspiration layer. No technical score measures or represents divine attributes, faith, spiritual rank, or divine approval. Name enumeration, Arabic, transliteration, and glosses require independent religious and language review.

## Evaluation

The implemented Core Integrity suite is offline and deterministic apart from report timestamp/platform metadata. It checks turn lifecycle and retry invariants, verified-XP gating and replay behavior, RID's 99 unrated slots, catalogue uniqueness, level thresholds, and explicit-memory baseline. The latest report is stored at `data/evaluations/core/latest.json` under the executable root.

External agent-trajectory, memory-transfer, and generalization benchmarks are planned or candidate integrations; they are not part of the current Core Integrity suite. Passing Core Integrity does not establish AGI.

## Planned architecture—not current capability

The workspace artifact command is a narrow first application service, not general agent tool access. It is currently invoked directly by the user; the planner does not yet propose or dispatch it. A version-1 JSONL trace records workspace proposal, approval/denial, execution/failure, and verification events without artifact bodies; complete turn/reflection persistence and restart recovery are not yet implemented. Filesystem checks fail closed for detected symlinks but do not claim protection against a hostile local process racing filesystem changes.

The following are future roadmap stages, not available seed-release behavior:

- Independent verification of plan acceptance checks against observed runtime effects.
- Typed application services and permissioned tool execution.
- Provider-aware retries, persisted turn traces, and user-interrupt wiring.
- Memory migrations, concurrency control, retention, retrieval ranking, and evaluated context injection.
- Read-only capability registry, sandboxed workspace tools, and human approval for side effects.
- Project builder, isolated candidate evolution, benchmarked promotion/rollback.
- Multi-agent workforce and desktop/web/API interfaces.

## Evolution and extension rules

Keep one Cargo package while it remains the simplest coherent unit. Split crates only when demonstrated ownership, compilation, or reuse needs justify it. Future interfaces and external runtimes must call shared core contracts; they must not fork identity, cognition, memory, or policy.

Any future self-evolution must operate on isolated candidates with bounded permissions, immutable baselines, reproducible tests/evaluations, review, lineage, and rollback. Generated code must not become trusted merely because it compiles.
