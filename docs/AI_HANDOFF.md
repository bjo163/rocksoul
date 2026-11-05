# Rocksoul AI Handoff Guide

**Purpose:** Give a local coding agent enough verified context to continue Rocksoul safely without relying on chat history.
**Repository:** `bjo163/rocksoul`
**Canonical branch:** `main`
**Last reviewed:** 2026-11-05
**Product principle:** **One Core. Infinite Evolution.**

> This document is a working handoff, not permission to bypass repository governance. The latest `origin/main` is always authoritative.

## 1. Start here — mandatory agent procedure

Before doing anything:

1. Read `AGENTS.md`, `blueprint.md`, `README.md`, `docs/PROGRESS.md`, this file, `docs/ARCHITECTURE.md`, `docs/COGNITIVE_CORE.md`, and `docs/ROADMAP.md`.
2. Fetch the latest `origin/main`. Record its exact SHA. Do not assume the current checkout is current.
3. Inspect open PRs and issues. Check whether the task is already implemented, in progress, superseded, or blocked.
4. Compare actual code and tests with docs. Treat code on latest `main` as the accepted implementation; a PR is only proposed work until merged.
5. Create a short-lived branch from latest `main`: `feat/*`, `fix/*`, `chore/*`, or `docs/*`.
6. Implement one coherent, testable vertical slice. Keep the diff small. Do not stack on another feature branch.
7. Run all relevant checks, update docs and roadmap in the same PR, and report exact evidence.
8. Open a PR targeting `main`. Do not merge unless required checks pass and the diff is reviewed.
9. After merge, fetch `main` again and verify its SHA includes the merge. Update local notes from that state.

If chat history, generated plans, issue descriptions, old branches, or this handoff disagree with latest `main`, latest `main` wins. Reconcile this document when implementation changes.

## 2. Product intent and non-negotiable boundaries

Rocksoul is a Rust-first, CLI-native intelligence runtime. It begins as one portable native executable and grows through tested, evidence-backed capabilities. AGI is a long-term research aspiration, not a demonstrated capability.

- One Cargo package and one native executable are the current product boundary.
- CLI/REPL is the first interface. Ratatui is a terminal cockpit over the same core, not a separate runtime.
- Keep cognition and policy independent from terminal presentation.
- Keep the core Rust-first. Do not introduce Python/Node runtimes as core dependencies.
- Do not add web, desktop, server, multi-package workspace, or multi-agent infrastructure without a demonstrated need and an explicit roadmap task.
- Model output and tool output are untrusted proposals, never proof that an action happened.
- No unrestricted shell execution, arbitrary filesystem mutation, browser/network/plugin execution, or autonomous code execution in the current seed runtime.
- No automatic memory capture or memory-context injection. Memory writes remain explicit and human-reviewed.
- Do not claim consciousness, AGI, general intelligence, or capabilities that are not implemented and tested.

### RID, 99 stats, XP and levels

Keep these concepts separate:

- **RID:** persistent local software identity and instance history.
- **Age:** elapsed time from the identity creation timestamp; not XP or level.
- **XP ledger:** append-only records for eligible verified work/events. Event identity must be idempotent; replay cannot award XP twice.
- **Level:** deterministic, versioned derivation from XP; never independently edited.
- **99 Stats:** 99 stable technical capability domains/slots, mapped one-to-one to the 99-name inspiration catalogue. The catalogue is a naming/navigation layer, not a claim about divine attributes.
- **Four Kitab context:** Al-Qur'an is the primary reference, with Taurat, Zabur, and Injil represented respectfully as the other three traditions in the product narrative. Do not turn religious text into a technical scoring system.

Never score or quantify divine attributes, faith, spiritual rank, religious worth, or divine approval. Technical stats measure only observable software capability and evidence. Preserve stable stat IDs and history across catalogue changes.

### Memory and truth provenance

Keep user statements, observations, inferences, and task outcomes distinct. Never promote a model inference or unsupported completion claim to verified fact. A kernel receipt is evidence of a bounded software verification event; it is not proof of broad real-world truth. A receipt digest is tamper-evident integrity metadata, not a cryptographic signature against a local actor with write access to the executable root.

### Portable-first data and security

Mutable configuration and runtime state belong under the executable/install root (portable root), not scattered into AppData or the user profile. Preserve user data on upgrades. Keep secrets in local environment/configuration excluded from Git and logs. Validate paths and fail closed. Use bounded timeouts/retries, cross-process locks for read-modify-write stores, backups, schema migrations, and explicit deletion/forgetting semantics.

## 3. Current repository shape

One Rust Cargo package (`Cargo.toml`, Rust 1.85 MSRV, edition 2024). Important modules:

| Module | Responsibility |
|---|---|
| `main.rs`, `lib.rs` | Process startup and library entry |
| `cli.rs`, `commands.rs` | Clap command tree and application commands |
| `core.rs` | Turn lifecycle, budgets, cancellation, evidence verification, XP gate, receipts |
| `model.rs` | OpenAI-compatible chat-completions adapter |
| `repl.rs` | Session-local REPL, bounded planner, model-only reasoning steps and reflector |
| `paths.rs` | Executable-root portable path resolution |
| `rid.rs`, `rid_store.rs`, `rid_catalogue.rs` | Identity, XP/level, stat slots and catalogue |
| `memory.rs` | Local memory store, schema migration, locking, retrieval, retention, forgetting |
| `memory_proposal.rs` | Pending model memory proposals and human approval |
| `workspace_artifact.rs` | Narrow create-only approved text artifact executor |
| `task_trace.rs` | Content-free JSONL task lifecycle traces |
| `eval.rs` | Offline deterministic Core Integrity evaluation |
| `tui.rs` | Ratatui Cognitive Cockpit |

Do not split crates or add layers just for theoretical future extensibility. Extract only when real ownership, compile-time, or reuse pressure justifies it.

## 4. Implemented baseline on main

PR #51 merged at `260d29852bb30f2f63129271a07cd368fafccc3f`. Issue #35 merged to `main` at `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`. Always fetch the latest `origin/main` before starting a new task.

### Stage 0 — seed CLI

- Rust CLI/REPL, OpenAI-compatible model adapter, local init/status/doctor.
- Portable executable-root layout and local environment configuration.
- RID profile, XP ledger, deterministic level progression, 99 unrated technical stat slots.
- Ratatui terminal cockpit.
- Offline Core Integrity evaluation and Windows x64 packaging/smoke tests.
- Manual clean-device install/upgrade/data-preservation test remains outstanding.

### Stage 1 — bounded reliable workspace artifact slice

PR #34 merged at `0f5ca4bc4d1c65322cb02963fcc23c53fa990f5d`; Linux/Windows CI #248 passed.

- Bounded planner and model-only reasoning; structured plan/reflection.
- One supported typed workspace-artifact operation, with exact human approval.
- Create-only UTF-8 `.txt` artifact publication and independent read-back verification.
- Bounded retries, cancellation checkpoints, content-free traces and persisted verification receipts.
- XP only after trusted verification; replay idempotency.
- Issue #35 / PR #53 is merged: bounded versioned `AcceptanceCheckSet` schema v1, deterministic post-publication crash recovery, fail-closed receipt conflict/corruption handling, and Core Integrity v1.3.0 checks are accepted on main.
- Still not implemented: full planner/turn/reflection conversation recovery across restart, acceptance-check adapters for additional typed services, arbitrary shell or unrestricted tool access.

### Stage 2 — persistent memory (merged main baseline)

The main baseline now includes local memory schema v4 and migrations from v1/v2/v3, cross-process locking, retention/forgetting, ranked lexical retrieval and offline evaluation, human-reviewed model proposals, structured claim provenance, task-bound kernel receipt verification, receipt revocation, and append-only claim supersession. Stage 2 memory implementation was extended by PR #51 (merge commit `260d29852bb30f2f63129271a07cd368fafccc3f`). Issues #37–#41, follow-up #44, and umbrella #36 are closed. Automatic conversation capture and automatic memory context injection remain disabled.

- Model proposals are staged separately and cannot silently write memory.
- Exact interactive approval binds the candidate fingerprint.
- Sensitive/credential-like content is rejected.
- Proposal candidate content is scrubbed from audit records after decision.
- Forgetting uses content-free tombstones and managed backup/export scrubbing.
- Retrieval is deterministic lexical ranking, not a probability.
- Automatic proposal emission from the REPL and automatic memory context injection remain disabled.

## 5. Recently completed — PR #51

**PR:** https://github.com/bjo163/rocksoul/pull/51  
**Issue:** #44 — closed after merge  
**Merge commit:** `260d29852bb30f2f63129271a07cd368fafccc3f`  
**PR head validated:** `817c651f23e3e74bd59c25c94585a7e2d3bf108b`  
**CI:** [run #42008862735](https://github.com/bjo163/rocksoul/actions/runs/42008862735) — Linux and Windows jobs passed.

Merged behavior:
- Structured claim provenance: user assertion, observed evidence, inference, unverified task outcome, verified task outcome, and contradicted/superseded.
- Memory schema v4 and safe v1/v2/v3 migrations without promoting legacy model proposals to verified status.
- Exact normalized claim, expected task/operation ID, verifier identity/version, evidence digest, outcome, timestamp and receipt integrity validation.
- Receipt-bearing proposals require both `--verification-receipt-id` and `--verification-task-id`; the expected task ID is included in the exact proposal approval fingerprint and must match the receipt's integrity-bound task ID and persisted operation ID.
- Freshness window (seven days), one-time receipt use, revocation ledger and downgrade of revoked verified claims.
- Append-only claim-status history for correction/supersession.
- Claim status and receipt provenance exposed through CLI and memory retrieval.
- Proposal store v1-to-v2 migration without weakening exact human approval.

### CI diagnosis and lessons retained

The initial Linux/Windows failure came from receipt lookup parsing lock files and unrelated files as receipts. Lookup now accepts only `.json` receipt files. A later review found that a valid receipt for the same text could otherwise be attached to a different task; the proposal now binds an expected task/operation ID, and regression tests cover cross-task mismatch and modified operation metadata.

The final validated head passed:
- Linux formatting, Clippy, 105 tests, Core Integrity and memory retrieval evaluation.
- Windows release build, tests, installer/portable packaging, clean portable smoke test and installer install/init/uninstall-preservation smoke test.

### Safety invariants now on main

- Model output cannot mint receipts or call the trusted verifier.
- User assertions do not need task receipts and remain explicitly labeled.
- Missing, stale, mismatched-claim, mismatched-task/operation, tampered, replayed or revoked receipts remain unverified.
- One receipt can verify at most one memory claim.
- Revocation is append-only and downgrades claims.
- Receipt digest is an integrity check, not a digital signature against a local actor with write access to the runtime root.
- Automatic conversation capture and memory context injection remain disabled.

## 6. Completed implementation — issue #35 / PR #53

**PR:** https://github.com/bjo163/rocksoul/pull/53 (merged)  
**Issue:** https://github.com/bjo163/rocksoul/issues/35 (closed)  
**Validated PR head:** `c9c605d6896930e5a36f30bb8d3fbe2ecfdd5484`  
**Merge commit on main:** `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`

The merged implementation contains:
- A private, test-only post-read-back failpoint; production passes a no-op and there is no CLI/environment/config switch.
- A shared workspace completion path that reloads RID after the failpoint, validates actual filesystem state, verifies evidence, persists XP, writes a receipt and appends the trace.
- Real-filesystem crash/replay coverage: artifact remains exact, XP is unchanged after injected process loss, replay returns AlreadyPresent, one XP event and one receipt are written, and a later replay is idempotent.
- Fail-closed immutable receipt conflict and malformed/truncated receipt preservation tests.
- `AcceptanceCheckSet` schema v1: max 64 checks; at least one required check; unique non-empty bounded IDs; bounded values/evidence references; unsupported versions rejected before XP mutation.
- Core Integrity v1.3.0 machine-readable checks for the new crash boundary, replay/XP/receipt/trace idempotency, receipt conflicts and corruption.
- Updated architecture, cognitive-core, roadmap, README, progress and plan documents in the merged PR.

Final CI [#42044132321](https://github.com/bjo163/rocksoul/actions/runs/42044132321) passed all Linux and Windows jobs on PR head `c9c605d6896930e5a36f30bb8d3fbe2ecfdd5484`. Linux passed formatting, Clippy, 108 tests, Core Integrity 23/23, and memory retrieval (precision@2 1.000, recall@2 1.000). Windows passed release build, 107 tests, installer/portable packaging, clean portable smoke, installer install/init/uninstall-preservation smoke, and artifact uploads. PR #53 was squash-merged to main as `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`; issue #35 is closed.

Optional temp-file/rename interruption injection was reviewed but not added: the current receipt publication uses create-only temporary file + sync + hard-link and has no existing deterministic seam. Do not introduce a production failpoint or broad filesystem abstraction solely for this optional subcase.

### Merged acceptance evidence
1. PR #53 is merged and issue #35 is closed.
2. Main was verified at merge SHA `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`.
3. Final CI #42044132321 is green on Linux and Windows, including packaging and installer/portable smoke.
4. Core Integrity v1.3.0 reports 23/23 checks passed; Linux has 108 tests and Windows has 107 tests.
5. Progress, roadmap, README and the Issue #9 execution plan are included in the post-merge documentation handoff.

## 7. Build, test and validate

Run from the repository root. Use the committed lockfile; do not silently update dependencies.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
cargo run --locked -- eval run core
cargo run --locked -- eval latest
```

For a fast focused test during development:

```powershell
cargo test --locked memory_proposal::tests::exact_fresh_kernel_receipt_is_required_for_verified_task_outcome
cargo test --locked memory_proposal::tests::receipt_is_one_time_and_cannot_verify_a_second_memory_claim
```

Before merge, require both CI paths:
- Linux: formatting, Clippy, full tests, Core Integrity, memory retrieval evaluation and report artifacts.
- Windows: release build, full tests, installer/portable packaging, clean portable smoke test and installer install/init/uninstall data-preservation smoke test.

A local green test is not a substitute for CI. A successful compile is not proof of correct behavior. Never merge while any required check is pending or failing.

## 8. Portable layout and configuration

The runtime resolves mutable state beside the executable. The initialized layout includes:

```text
Rocksoul/
├── rocksoul[.exe]
├── .env
├── config/config.toml
├── data/
│   ├── identity/rid.json
│   ├── memory/
│   └── evaluations/core/latest.json
├── cache/
├── workspace/
├── evolution/
└── logs/
```

Example local model endpoint:

```dotenv
ROCKSOUL_BASE_URL=http://localhost:11434/v1
ROCKSOUL_MODEL=your-installed-model
ROCKSOUL_API_KEY=
```

Use HTTPS for remote endpoints; plain HTTP is reserved for loopback model servers. Process environment values take precedence over root-local `.env`. Never add real credentials to fixtures, test output, docs, or commits.

## 9. GitHub workflow and handoff completion criteria

- `main` is the sole source of truth.
- Branch from latest `main`; never stack branches or target a feature branch.
- Prefix branches `feat/`, `fix/`, `chore/`, `docs/`.
- Keep one coherent vertical slice per PR.
- Include tests, docs, roadmap status, risks, and exact validation evidence.
- After merge, verify PR state, merge SHA, latest main ancestry/content, issue closure, and release status if applicable.
- Do not close an issue merely because a PR exists. Close only after the relevant work is merged and verified.
- Do not create a release unless requested or required by the approved roadmap.

A task is complete only when:
1. Implementation matches the accepted blueprint and does not cross safety boundaries.
2. Tests cover success, failure, replay, migration, corruption and relevant platform-specific behavior.
3. Docs and roadmap match the code actually merged to main.
4. Linux and Windows CI pass.
5. PR is merged to main and main is re-checked.
6. Related issue is closed with the merge evidence.
7. The next task is recorded with dependencies and acceptance criteria.

## 10. Next task selected — issue #9 Cognitive Cockpit completion

Issue #35 is complete. The next selected implementation task is [#9 — Cognitive Cockpit completion](https://github.com/bjo163/rocksoul/issues/9), based on the existing TUI rather than a new interface.

Execution plan: [spec](plans/issue-9/spec.md), [design](plans/issue-9/design.md), [small autonomous tasks](plans/issue-9/task.md), and [agent rules](plans/issue-9/rules.md).

Audit findings from main:
- Eight screens, keyboard navigation, command palette/help and Ratatui terminal lifecycle already exist.
- Overview/Brains/Tasks/Tools/Settings are partly static; no specialist registry or tool registry exists, so the UI must not invent roles or tools.
- Tasks need persisted content-free task-trace history; no live task queue is implemented.
- Memory is summary-only; add bounded search/filter/detail with content hidden by default.
- Diagnostics should include actual portable store health and latest Core Integrity report.
- Settings need portable presentation preferences and real dark/light theme handling.
- Only two TUI navigation tests exist; add read-model, privacy, error, responsive and preference tests.
- The Memory screen still displays `store.v1.json`; the actual store is `store.v4.json`.

Issue #13 owns the broader AGI-oriented evaluation harness and explicitly depends on #4, #5, #8 and #9. Do not duplicate or start that full harness before dependencies are audited. Issue #8 already has a 99-entry catalogue but remains open for independent religious/language review; AI self-review is not sufficient. Keep memory context injection disabled until separate policy, contamination, privacy, forgetting/deletion and regression gates are reviewed.

For every new task, write/update an execution plan with:
- Goal and explicit non-goals.
- Current main SHA and source files.
- Invariants and threat model.
- Small numbered implementation tasks.
- Test cases, including negative/replay/migration cases.
- Documentation files to update.
- Acceptance criteria and exact CI gates.
- Rollback/recovery notes.
- Handoff summary for the next agent.

## 11. Known deferred work (do not mark complete prematurely)

- Full planner/turn/reflection conversation resume across process restarts.
- Independent acceptance-check adapters for additional typed services; schema v1 contract alone does not implement those services.
- TUI completion work described in issue #9 and its execution plan.
- Broader memory injection policy and contamination/regression evaluation.
- Manual clean-device Windows install/upgrade verification.
- Typed read-only tool registry, sandboxing and permission/audit model.
- Project builder and isolated candidate evolution with evaluation, approval and rollback.
- Multi-agent delegation and additional interfaces.

---

**Handoff rule:** If uncertain, inspect the code and tests on latest `main`, state what is unknown, and fail closed. Never convert assumptions into completion claims.
