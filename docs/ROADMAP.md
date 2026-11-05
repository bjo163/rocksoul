# Rocksoul Roadmap

The roadmap is capability-driven. Each stage must deliver a working, tested increment. The stages are not promises of AGI.

## Stage 0 — Seed CLI

**Status:** Seed implementation, committed dependency lockfile, and automated clean Windows-runner smoke tests are verified by CI. Manual clean-device verification remains required.

- [x] One Rust Cargo project and native executable.
- [x] Interactive CLI with help, status, clear, and exit.
- [x] OpenAI-compatible chat-completions adapter.
- [x] Local bootstrap and diagnostics commands.
- [x] Windows x64 release build and CI artifact.
- [x] RID local identity profile, age, XP ledger and 99 unrated stat slots.
- [x] Keyboard-first Ratatui Cognitive Cockpit.
- [x] Commit the resolved Cargo.lock; CI and release builds use locked dependency resolution.
- [x] Verify installer and portable ZIP on a clean Windows runner, including init, re-init preservation, CLI diagnostics, core evaluation, and memory operations.
- [ ] Perform one manual clean-device installation and verify upgrade/data preservation.

## Stage 1 — Reliable cognition loop

**Status:** The Stage 1A–1E bounded workspace-artifact vertical slice is merged to `main` via PR #34 (`0f5ca4bc4d1c65322cb02963fcc23c53fa990f5d`). Linux/Windows CI #248 passed. The delivered slice includes exact human approval, create-only execution, observed read-back verification, bounded retries/cancellation, versioned traces, persisted receipts, and XP replay idempotency. This does not mean the broader autonomous plan/act/observe loop is complete; residual planner recovery and broader acceptance-check orchestration are explicitly deferred to [#35](https://github.com/bjo163/rocksoul/issues/35).

- [x] Explicit request and turn-state contracts.
- [x] Structured plan and reflection data contracts.
- [x] Cooperative cancellation token.
- [x] Model-call, output-token and wall-clock budget meter.
- [x] Structured JSON planning with validated unique steps and required acceptance checks.
- [x] Bounded model-only execution for up to three ordered reasoning steps.
- [x] Structured reflector review with explicit limitation that it is not independent evidence.
- [x] Five-call, 16,000-token, 120-second per-turn budget; per-call token reservations remain bounded even when usage metadata is absent.
- [x] Deterministic lifecycle, retry, terminal-state and budget tests.
- [x] Independent evidence verifier contract for XP; failed required checks award no XP and replay is idempotent.
- [x] Allow the bounded planner to return an optional typed workspace-artifact intent; validate it and dispatch only through the exact human-approval CLI boundary.
- [x] Deliver one approved typed workspace-artifact operation through the bounded planner/CLI approval boundary, with independent read-back evidence and persisted receipt.
- [ ] Generalize plan acceptance checks across multiple typed services (deferred; do not introduce unrestricted tools).
- [x] Add a three-attempt, bounded-backoff retry wrapper for transient workspace filesystem errors; create-only replay remains idempotent.
- [x] Derive a stable operation/XP idempotency key from the exact filename and content pair.
- [x] Add provider-aware model retry classification for connection/timeout and HTTP 408/425/429/5xx failures; every attempt consumes the shared turn call budget.
- [x] Wire Ctrl+C during model-driven turns into the cooperative cancellation token.
- [x] Wire Ctrl+C through terminal approval and executor; cancellation is honored at safe pre-publication checkpoints and in-flight model HTTP requests.
- [x] Add deterministic cancellation-at-publication-boundary coverage; cancellation before the irreversible hard-link publish leaves no artifact.
- [x] Test restart/replay around XP and receipt persistence windows; preserve idempotent XP and durable receipt reconstruction.
- [ ] Add a crash-injection test for recovery after file publication but before XP persistence.
- [x] Verify the supported workspace-artifact task against observed file metadata/read-back; chat response completion is not task success.
- [ ] Extend independent acceptance verification to future typed services as each is introduced.
- [x] Append content-free JSONL lifecycle events for the workspace CLI (proposal, approval/denial, execution/failure, verification).
- [x] Persist create-only, versioned verification receipt JSON files and expose them through trace inspection.
- [x] Serialize JSONL trace appends across processes with portable OS file locks and validate schema version/required fields (Rust 1.85 MSRV preserved).
- [x] Extend offline Core Integrity v1.2 with workspace proposal/approval, create/read-back, verified XP, replay idempotency, pre-cancel, trace round-trip, and Unix target-symlink scenarios.
- [x] Cover supported artifact approval, read-back, mismatch, replay, cancellation boundary, trace, and platform symlink cases in deterministic tests/CI.
- [ ] Expand the evaluation matrix for future typed services and additional crash-injection points.
- [ ] Persist complete planner/turn/reflection conversation state and resume interrupted model reasoning (deferred; current recovery replays the bounded operation and durable receipt, not model conversation history).

## Stage 2 — Persistent memory

**Status:** Local-only schema v3 memory lifecycle and deterministic retrieval evaluation are merged to `main`. The human-reviewed proposal workflow is being validated in PR #50; automatic cognition-to-memory ingestion and retrieval injection remain disabled.

- [x] Add a root-local JSON store with schema versioning and atomic replacement/backup recovery.
- [x] Separate episodic, semantic, procedural, working, and derived record kinds.
- [x] Require explicit user CLI action for every write; model output cannot save or delete memory.
- [x] Support list, text/tag search, permanent deletion, confirmed clear, and root-local export.
- [x] Support expiry for working-memory records.
- [x] Show real store counts and provenance in the Cognitive Cockpit.
- [x] Migrate v1-to-v2-to-v3 memory stores with retention metadata, metadata-preserving validation, and backup recovery diagnostics.
- [x] Serialize memory read-modify-write CLI transactions with a stable OS lock file, bounded five-second acquisition timeout, and subprocess tests for contention, process-exit release, and concurrent writers.
- [x] Add versioned per-record retention policies, dry-run-first expiry pruning, and append-only content-free forgetting tombstones.
- [x] Apply tombstones during normal load and backup recovery; scrub managed backups, legacy snapshots, and exports after explicit forgetting.
- [x] Add deterministic lexical ranking, provenance filters, and offline held-out synthetic retrieval evaluation with precision@k/recall@k, stale ranking, expiry leakage, and deletion leakage gates.
- [x] Persist machine-readable retrieval reports and run the evaluation in Linux CI and Windows portable smoke tests.
- [ ] Consider memory context injection only after separate policy approval and additional contamination/regression tests.
- [x] Add typed model-proposal records with source-turn provenance, retention suggestions, credential/sensitive-category rejection, exact SHA-256 payload binding, one-time interactive approval, denial/cancellation, and idempotent persistence; silent model writes and automatic context injection remain disabled.

## Stage 3 — Skills and tools

- [ ] Define a typed tool contract and capability registry.
- [ ] Start with read-only, narrowly scoped tools.
- [ ] Add explicit permission checks, audit events, and timeouts.
- [ ] Add workspace sandboxing and human approval for writes or remote egress.
- [ ] Keep external runtimes optional rather than core dependencies.

## Stage 4 — Application builder

- [ ] Create projects from explicit templates.
- [ ] Work only inside an explicitly selected workspace.
- [ ] Track generated files and changes.
- [ ] Run formatters, compilers, tests, and project-specific checks.
- [ ] Report what changed and what was not completed.

## Stage 5 — Evaluation and isolated evolution

- [x] Offline deterministic Core Integrity suite and JSON report.
- [x] Versioned external benchmark candidate manifest; no external suite is claimed as integrated.
- [ ] Add sandboxed agent trajectory and tool-use task suites.
- [ ] Add memory retention, forgetting and held-out transfer suites.
- [ ] Validate benchmark licenses, access, cost, environment and contamination risk before adapter integration.
- [ ] Create candidate branches/worktrees and immutable baselines.
- [ ] Compare candidate results with matched baseline conditions.
- [ ] Preserve candidate lineage, logs, artifacts, and evaluation results.
- [ ] Require approval before promoting core changes.
- [ ] Support rollback and reproducible rebuilds.

## Stage 6 — Multi-agent capabilities

- [ ] Add delegation only when single-agent workflows show a measurable need.
- [ ] Give each worker bounded tasks, tools, context, and budgets.
- [ ] Add coordination, shared artifacts, conflict handling, and evaluation.

## Stage 7 — Additional interfaces

- [ ] Add desktop, web, API, or automation interfaces as separate presentation/adaptor layers.
- [ ] Keep the same core contracts and identity.
- [ ] Do not duplicate cognition or memory logic per interface.

## Evaluation interpretation

Rocksoul is an AGI-oriented research and engineering project. It has not demonstrated AGI, consciousness, or general intelligence. Do not publish one scalar AGI score. Report versioned capability vectors, evidence, uncertainty, sample counts and safety gates. Safety/privacy/permission failures are vetoes and cannot be averaged away. See [AGI Capability Evaluation](evaluation/AGI-capability-evaluation.md).

## Release gate

A stage is complete only when implementation, tests, documentation, security review, and reproducible build evidence are present. Roadmap checkboxes must reflect actual repository state.
