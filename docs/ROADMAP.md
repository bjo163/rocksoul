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

**Status:** The bounded workspace-artifact slice from PR #34 is extended by Issue #35 / PR #53, merged as `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`; issue #35 is closed. The merged reliability work adds deterministic recovery after publication but before XP persistence, immutable receipt conflict/corruption handling, and a versioned bounded `AcceptanceCheckSet` schema v1 consumed by the trusted verifier. Core Integrity v1.3.0 covers both crash windows and receipt conflict/corruption; CI #42044132321 passed on Linux and Windows. Full planner/reflection conversation resume and independent acceptance adapters for additional typed services remain deferred.

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
- [x] Add a private test-only crash-injection test after file publication/read-back but before XP persistence; replay re-verifies the existing artifact and records XP exactly once.
- [x] Reject conflicting immutable and malformed durable receipts without overwriting or deleting their bytes.
- [x] Define bounded, versioned `AcceptanceCheckSet` schema v1 (max 64 checks, unique IDs, required-check presence, bounded values/evidence references, unsupported-version rejection).
- [x] Extend Core Integrity v1.3.0 with machine-readable crash-window, replay/idempotency, receipt-conflict and receipt-corruption checks.
- [x] Verify the supported workspace-artifact task against observed file metadata/read-back; chat response completion is not task success.
- [ ] Extend independent acceptance verification to future typed services as each is introduced.
- [x] Append content-free JSONL lifecycle events for the workspace CLI (proposal, approval/denial, execution/failure, verification).
- [x] Persist create-only, versioned verification receipt JSON files and expose them through trace inspection.
- [x] Serialize JSONL trace appends across processes with portable OS file locks and validate schema version/required fields (Rust 1.85 MSRV preserved).
- [x] Extend offline Core Integrity v1.2 with workspace proposal/approval, create/read-back, verified XP, replay idempotency, pre-cancel, trace round-trip, and Unix target-symlink scenarios.
- [x] Extend Core Integrity v1.3.0 with post-publication crash recovery, replayed XP/receipt/trace idempotency, immutable receipt conflict and malformed receipt preservation (PR #53, issue #35).
- [x] Cover supported artifact approval, read-back, mismatch, replay, cancellation boundary, trace, and platform symlink cases in deterministic tests/CI.
- [ ] Expand the evaluation matrix for future typed services and additional crash-injection points.
- [ ] Persist complete planner/turn/reflection conversation state and resume interrupted model reasoning (deferred; current recovery replays the bounded operation and durable receipt, not model conversation history).

## Stage 2 — Persistent memory

**Status:** Stage 2's explicit local memory lifecycle is implemented through schema v4, including migrations, locking, retention/forgetting, evaluated retrieval, human-reviewed proposals, structured claim provenance, task-bound kernel receipt gating, revocation and supersession. PR #51 merged as `260d29852bb30f2f63129271a07cd368fafccc3f`; issue [#44](https://github.com/bjo163/rocksoul/issues/44) is closed. Automatic cognition-to-memory emission and retrieval injection remain disabled pending separate policy and contamination gates.

- [x] Add a root-local JSON store with schema versioning and atomic replacement/backup recovery.
- [x] Separate episodic, semantic, procedural, working, and derived record kinds.
- [x] Require explicit user CLI action for every write; model output cannot save or delete memory.
- [x] Support list, text/tag search, permanent deletion, confirmed clear, and root-local export.
- [x] Support expiry for working-memory records.
- [x] Show real store counts and provenance in the Cognitive Cockpit.
- [x] Migrate v1-to-v2-to-v3-to-v4 memory stores with retention and claim-provenance metadata, metadata-preserving validation, and backup recovery diagnostics.
- [x] Gate verified task outcomes on exact fresh single-use kernel receipts with a matching expected task/operation ID; preserve user assertions, support receipt revocation downgrade and append-only supersession (PR #51, issue #44).
- [x] Serialize memory read-modify-write CLI transactions with a stable OS lock file, bounded five-second acquisition timeout, and subprocess tests for contention, process-exit release, and concurrent writers.
- [x] Add versioned per-record retention policies, dry-run-first expiry pruning, and append-only content-free forgetting tombstones.
- [x] Apply tombstones during normal load and backup recovery; scrub managed backups, legacy snapshots, and exports after explicit forgetting.
- [x] Add deterministic lexical ranking, provenance filters, and offline held-out synthetic retrieval evaluation with precision@k/recall@k, stale ranking, expiry leakage, and deletion leakage gates.
- [x] Persist machine-readable retrieval reports and run the evaluation in Linux CI and Windows portable smoke tests.
- [ ] Consider memory context injection only after separate policy approval and additional contamination/regression tests.
- [x] Add typed model-proposal records with source-turn provenance, retention suggestions, credential/sensitive-category rejection, exact SHA-256 payload binding, one-time interactive approval, denial/cancellation, post-decision content scrubbing, and idempotent persistence; silent model writes and automatic context injection remain disabled.


## Next implementation dependency

The next selected task is [Issue #9 — Cognitive Cockpit completion](https://github.com/bjo163/rocksoul/issues/9). The initial eight-screen TUI exists, but persisted task history, searchable memory browsing, diagnostics/evaluation reporting, portable presentation preferences and deeper test coverage remain incomplete. See `docs/plans/issue-9/` for the autonomous execution plan.

Issue #13's AGI-oriented evaluation harness depends on #4, #5, #8 and #9. Do not duplicate or begin the full harness until its dependencies are audited and closed. Issue #8's catalogue implementation exists, but its independent religious/language review remains outstanding and must not be self-approved by an AI model.

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

- [x] Offline deterministic Core Integrity v1.3.0 suite and JSON report.
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
