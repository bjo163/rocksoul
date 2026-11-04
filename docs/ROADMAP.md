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

**Status:** Core contracts implemented; task-level orchestration remains in progress

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
- [ ] Verify plan acceptance checks against independent runtime evidence.
- [ ] Execute plans through typed application services and approved tools.
- [ ] Add provider-aware retry classification, backoff and idempotency.
- [ ] Wire user interruption into cooperative cancellation.
- [ ] Verify task acceptance against observed effects; chat response completion is not task success.
- [ ] Persist turn traces and reflection records.

## Stage 2 — Persistent memory

**Status:** First explicit, local-only memory store implemented; automated cognition-to-memory remains disabled.

- [x] Add a root-local JSON store with schema versioning and atomic replacement/backup recovery.
- [x] Separate episodic, semantic, procedural, working, and derived record kinds.
- [x] Require explicit user CLI action for every write; model output cannot save or delete memory.
- [x] Support list, text/tag search, permanent deletion, confirmed clear, and root-local export.
- [x] Support expiry for working-memory records.
- [x] Show real store counts and provenance in the Cognitive Cockpit.
- [ ] Add explicit schema migrations and backup restore diagnostics.
- [ ] Add memory locking for multiple concurrent Rocksoul processes.
- [ ] Add retention policies, forgetting workflows, and cross-session evaluation.
- [ ] Add retrieval ranking and memory context injection only after provenance and relevance tests.
- [ ] Add model-proposed memory with human approval; never enable silent model writes.

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
