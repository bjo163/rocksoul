# Rocksoul AI Handoff Guide

**Purpose:** Give a local coding agent enough verified context to continue Rocksoul safely without relying on chat history.
**Repository:** `bjo163/rocksoul`
**Canonical branch:** `main`
**Last reviewed:** 2026-11-06
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
- **99 Universal Stats:** the canonical technical taxonomy is `data/rid/universal_stats.v2.json`: 99 observable capability dimensions across 99 independent first-class domains, each with a stable `rid_stat:NN` ID, action-word label, definition, and evidence-oriented evaluation protocol. Scores begin unrated and require trusted external evaluation.
- **Asmaul Husna reference:** `data/rid/asmaul_husna.v1.json` is a separate optional religious/inspirational catalogue. It does not define stat identities, labels, scores, or evaluation. Never infer semantic or theological equivalence from matching ordinal positions. Religious/language review remains a human expert gate.
- **Four Kitab context:** Al-Qur'an is the primary reference, with Taurat, Zabur, and Injil represented respectfully as the other three traditions in the product narrative. Do not turn religious text into a technical scoring system.

Never score or quantify divine attributes, faith, spiritual rank, religious worth, or divine approval. Technical stats measure only observable software capability and evidence. Preserve stable stat IDs and history across catalogue changes.

### Memory and truth provenance

Keep user statements, observations, inferences, and task outcomes distinct. Never promote a model inference or unsupported completion claim to verified fact. A kernel receipt is evidence of a bounded software verification event; it is not proof of broad real-world truth. A receipt digest is tamper-evident integrity metadata, not a cryptographic signature against a local actor with write access to the executable root.

### Portable-first data and security

Mutable configuration and runtime state belong under the executable/install root (portable root), not scattered into AppData or the user profile. Preserve user data on upgrades. Keep secrets in local environment/configuration excluded from Git and logs. Validate paths and fail closed. Use bounded timeouts/retries, cross-process locks for read-modify-write stores, backups, schema migrations, and explicit deletion/forgetting semantics.


### Universal 99 Stats — canonical taxonomy and migration guard

- Canonical taxonomy file: `data/rid/universal_stats.v2.json`, catalogue version `2.3.3`.
- Exactly 99 first-class independent universal capability domains, each linked to exactly one stat. There is no mandatory parent-domain hierarchy and no nine-domain × eleven-stat grouping. Never double-count one observed outcome across multiple stats; multi-stat evaluation requires distinct outcome evidence and separate rubric checks. Domain IDs are `rid_domain:01`–`rid_domain:99`; stat IDs are `rid_stat:01`–`rid_stat:99`.
- Every domain has a unique key, name, primary measurement target, inclusion/exclusion criteria, adjacent-overlap IDs, and a unique domain-specific separation rule; each linked stat's evaluation protocol embeds that rule. Domain definitions and linked stat slots are one-to-one representations of the same 99 constructs, not 198 capabilities: domain = construct definition; stat = per-profile measurement slot. The graph contains 322 directed overlap guards / 161 reciprocal pairs. The JSON includes 161 canonical pairwise contrast records (`rid_overlap:001`–`rid_overlap:161`) with both constructs' measurement targets and explicit separate-outcome rules. Runtime validation enforces exact graph/registry parity, case-insensitive uniqueness, normalized keys, sorted unique reciprocal edges, graph counts, and domain/stat metadata parity. The former v1 nine-by-eleven taxonomy is historical/superseded. Taxonomy overlap boundaries are explicit and structurally validated; independent psychometric validation and trusted evaluator integration remain separate implementation tasks. Correlation is expected; double-counting is prohibited.
- The new catalogue IDs are `rid_stat:01` through `rid_stat:99`. Rust runtime now uses `rid_stat:01` through `rid_stat:99` and embeds universal catalogue v2.3.3. Schema-v1 profiles migrate by preserving exact original bytes in `profile.json.v1.bak`, retaining all 99 legacy records in an immutable archive, preserving identity/XP, and initializing new universal stats as unrated. Never map legacy scores by ordinal position. Migration and tests are on main; verify CI before declaring completion.
- Migration must be explicit, versioned, backed up, reversible/recoverable, and tested with legacy profile fixtures. Update Rust domain, persistence, CLI, Cockpit, docs, and tests as one coherent slice.
- Keep all scores unrated until trusted evidence exists. `not_applicable` is not score zero. XP, level, and age are not proxies for stat capability.
- Previous CI diagnosis for the v2.3.3 slice: formatting drift was corrected in the affected Rust files; the Windows migration-test fixture now uses owned `String` evidence references; and a redundant read-only archive `sync_all` that caused Windows `Access is denied` after the archive data had already been synced was removed. Exact-main CI is still pending; verify both Linux and Windows jobs before closing Issue #5.
- Do not close the religious/language review gate for the separate Asmaul Husna catalogue without qualified independent reviewers.

- Current catalogue refinement: v2.3.3; stat 42 is RESUME (stable ID retained) to distinguish interruption recovery from long-run reliability. The ten cognitive families in the evaluation plan are benchmark coverage tags only, never parent domains or extra RID scores. `data/evaluation/benchmark-manifest.v1.json` v1.1.4 binds adapters to the RID v2.3.3 registry and carries the no-double-counting rule.

- The second-pass review added 12 adjacency guards, including READ/UNDERSTAND, READ/TRANSLATE, TRACK/DIALOGUE, LEARN/INTEGRATE, OPTIMIZE/ITERATE, COMPLETE/SELF_CHECK, EXPRESS/STRUCTURE, TRANSLATE/TUNE, BOUNDARY/SECURE, SECURE/RESIST, and GOAL_TRACK/HANDOFF.
- Third-pass review added 12 more guards: NOTICE/ERROR_DETECTION, TRACK/MONITOR, TRACK/GOAL_TRACK, UPDATE/SYNTHESIZE, ANALYZE/REFLECT, PRIORITIZE/ESCALATE, EXECUTE/TRANSFER, EXECUTE/MONITOR, ALIGN/VIEWPOINT_MODEL, ALIGN/INCLUDE, INVENT/REFRAME, and EXPERIMENT/REFLECT.

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
| `eval.rs` | Offline deterministic Core Integrity and memory-retrieval evaluation; JSON reports include source revision |
| `tui.rs` | Ratatui Cognitive Cockpit |

Do not split crates or add layers just for theoretical future extensibility. Extract only when real ownership, compile-time, or reuse pressure justifies it.

## 4. Implemented baseline on main

PR #51 merged at `260d29852bb30f2f63129271a07cd368fafccc3f`. Issue #35 merged to `main` at `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`. Always fetch the latest `origin/main` before starting a new task.

### Stage 0 — seed CLI

- Rust CLI/REPL, OpenAI-compatible model adapter, local init/status/doctor.
- Portable executable-root layout and local environment configuration.
- RID profile, XP ledger, deterministic level progression, 99 unrated technical stat dimensions across 99 independent domains.
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

Build metadata: `build.rs` exposes `ROCKSOUL_GIT_SHA` to the binary. CI sets it from `${{ github.sha }}`; local builds without the variable report `unknown`. Both deterministic report types include a backward-compatible `rocksoul_commit` field. Core Integrity and Memory Retrieval each persist canonical `latest.json` plus a human-readable `latest.md`; CI archives all four files. The CLI prints the source revision. Model-dependent trajectory/learning reports are not yet implemented.

## 10. Next task selected — Issue #13 technical evaluation harness

Issue #35 and Issue #9 are complete. Issue #9's Cognitive Cockpit was merged by PR #65 as `6474cd130bd8146f03dcacb757d9c341d4b1161a`, and Issue #9 is closed. The next engineering task is [Issue #13](https://github.com/bjo163/rocksoul/issues/13): make evaluation runs reproducible, source-identifiable, and auditable across deterministic and model-dependent suites. The separate [Issue #8](https://github.com/bjo163/rocksoul/issues/8) religious/language review remains a human-expert governance task and is not a technical prerequisite for the evaluation harness; no AI self-approval is allowed for that review.

Completed execution record: [Issue #9 spec](plans/issue-9/spec.md), [design](plans/issue-9/design.md), [task ledger](plans/issue-9/task.md), and [agent rules](plans/issue-9/rules.md).

Audit findings from main:
- Eight screens, keyboard navigation, command palette/help and Ratatui terminal lifecycle already exist.
- Overview/Brains/Tasks/Tools/Settings are partly static; no specialist registry or tool registry exists, so the UI must not invent roles or tools.
- Tasks need persisted content-free task-trace history; no live task queue is implemented.
- Memory is summary-only; add bounded search/filter/detail with content hidden by default.
- Diagnostics should include actual portable store health and latest Core Integrity report.
- Settings need portable presentation preferences and real dark/light theme handling.
- Only two TUI navigation tests exist; add read-model, privacy, error, responsive and preference tests.
- The Memory screen still displays `store.v1.json`; the actual store is `store.v4.json`.

Issue #13 depends technically on the Cognitive Kernel (#4), RID implementation/validation (#5), and Cognitive Cockpit read-only integration (#9). Issue #8 is deliberately decoupled: its religious/language expert review does not block technical benchmark infrastructure because the Asmaul Husna reference is not part of the scoring taxonomy. Keep memory context injection disabled until separate policy, contamination, privacy, forgetting/deletion and regression gates are reviewed.

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

## 12. Completed handoff — Issue #9 Cognitive Cockpit

**Issue:** [#9 Cognitive Cockpit](https://github.com/bjo163/rocksoul/issues/9) — closed  
**PR:** [#65 — feat(tui): complete Rocksoul Cognitive Cockpit](https://github.com/bjo163/rocksoul/pull/65) — merged  
**Validated PR head:** `383d8cfa3998105d34f6897b93bc14bce5329a55`  
**Merge commit / verified main SHA:** `6474cd130bd8146f03dcacb757d9c341d4b1161a`  
**Validation:** [CI #42071047960](https://github.com/bjo163/rocksoul/actions/runs/42071047960) — Linux and Windows passed.

### Merged capabilities

- Typed snapshot projections load RID, model configuration presence, memory store/health, bounded task history, Core Integrity and memory retrieval reports, and presentation preferences once; render functions are read-only.
- Task Monitor reads a bounded content-free JSONL trace (1 MiB ceiling), newest-first, with keyboard selection/detail and sanitized metadata only.
- Memory browser supports bounded local search and kind/claim/source filters, provenance/approval/expiry metadata, and explicit Enter-to-reveal through the shared sensitive-content guard. Content remains hidden by default and is clipped/sanitized when revealed.
- Diagnostics report memory-store health and latest offline evaluation states without executing evaluations during render; errors are sanitized.
- Versioned portable preferences support dark/light theme, compact/comfortable density and English/Indonesian locale. Missing files use defaults; malformed and unknown schemas are preserved rather than overwritten.
- Semantic palettes and locale are applied across screens/overlays. TestBackend covers all eight screens at 80×24, 120×40 and 60×20, both themes/locales, overlays and keyboard transitions.
- No model/backend calls, tool invocation, remote egress, task execution, memory mutation, automatic capture or context injection originates from the Cockpit.

### Exact-head validation evidence

- Linux CI #42071047960: formatting and Clippy passed; 130 tests passed; Core Integrity 23/23; memory retrieval precision@2 1.000 and recall@2 1.000.
- Windows CI #42071047960: release build and 129 tests passed; Core Integrity 22/22; memory retrieval precision@2 1.000 and recall@2 1.000; installer/portable packaging, clean portable smoke, installer install/init/uninstall-preservation smoke and artifact uploads passed.
- Full 13-file diff reviewed against Issue #9 and safety invariants. Earlier CI defects (rustfmt, missing Palette argument, unused/recursive tool-registry helper) were fixed before validation.

### Integration verified

1. PR #65 was marked ready and squash-merged into `main`.
2. `main` was fetched after merge and verified at `6474cd130bd8146f03dcacb757d9c341d4b1161a`.
3. Issue #9 was closed as completed with merge and CI evidence.
4. README, progress, roadmap, terminal UI guide, architecture and task ledger were synchronized after merge.

### Next dependency-safe task

Issue #8 still requires independent human religious/language review of the optional reference catalogue; do not self-approve this review with an AI model. Issue #13's technical evaluation work can proceed independently of #8, but trusted RID score publication remains gated on #5 passing exact-head CI and on independent evaluator validation.

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


## Issue #13 — trajectory evidence handoff (2026-11-06)

Latest implementation on `main` adds:
- Typed, bounded trajectory metadata/evidence contracts in `src/eval.rs`.
- Public deny-by-default tool metadata registry in `src/tool_registry.rs`: explicit tool-ID allowlist, separate capability grants, duplicate/invalid descriptor rejection, and descriptor/policy input-byte budgets.
- Deterministic in-process fixture executor in `src/fixture_runtime.rs`: only bounded key-value read/write over a private memory map, capped at 16 calls, 16 KiB aggregate input, 32 keys and 4 KiB per value; no shell, network, filesystem, process, environment-variable, or arbitrary-code capability.
- Fixture verification reads executor-observed state and rejects forged evidence references. It deliberately marks its verifier as **not independent**; it is not eligible to establish a model trajectory success or RID score.
- Seven fixture-runtime tests cover default-deny, observed-state verification, read-after-write, missing capability, unknown tool/evidence, and path-like key rejection.
- Benchmark manifest v1.1.5 registers `rocksoul.agent-runtime-fixtures` as offline deterministic CI tests. The actual model-dependent `rocksoul.agent-trajectory` remains `planned`.

**Important boundary:** this fixture runtime is not an OS sandbox and does not execute model-selected arbitrary tools. The model adapter remains chat-completion-only. Production OS isolation, independently operated verifier adapters, held-out task execution, trajectory report persistence, trusted RID publication, and candidate-vs-baseline rollback remain open. Next order: (1) threat-model and select an OS isolation backend, (2) implement bounded executor with hard wall-clock/CPU/memory/process/network/filesystem limits, (3) verifier adapters that observe effects outside the model/executor trust boundary, (4) held-out dataset/task partition and leakage controls, (5) model-connected trajectory runner/report persistence, (6) trusted evaluator and regression/rollback gates.


## Production sandbox threat model

See [docs/plans/issue-13/sandbox-threat-model.md](plans/issue-13/sandbox-threat-model.md) before implementing any production executor. It defines trust boundaries, no-WASI/no-network defaults, OS worker isolation, resource ceilings, verifier separation, adversarial tests, and the MSRV gate for Wasmi. Do not bypass these gates by reusing the in-process fixture executor.

### Issue #13 — independent verifier protocol handoff

Read [docs/plans/issue-13/independent-verifier-contract.md](plans/issue-13/independent-verifier-contract.md) together with the sandbox threat model before implementing production trajectory verification. The contract requires a host-created task-attempt envelope and single-use challenge nonce; an independently controlled observation adapter; authenticated, task-bound attestation; replay/freshness checks; three-valued pass/fail/indeterminate semantics; and real process/OS isolation tests. A mock, model self-report, guest tool output, or in-process fixture is not independent evidence. If platform isolation or verifier availability is missing, fail closed or report unsupported/indeterminate. This document is design-only and does not authorize marking the trajectory suite implemented.


## Issue #67 — merged tool audit and fixture invocation boundary (2026-11-06)

PR #69 is merged to `main` as `c845a9485a9b1b2882691c8dc268d6e6385b6be2`. Its exact implementation head `5151e47abfde528e95b4a0ded78e1469aacd7860` passed CI run [#42211510844](https://github.com/bjo163/rocksoul/actions/runs/42211510844) on Linux and Windows.

- Read `src/tool_audit.rs` for the closed content-free event schema, locked append-only JSONL store, fail-closed corruption behavior, and cooperative invocation controls.
- Read `src/fixture_runtime.rs` for audited fixture dispatch and pre-dispatch deadline/cancellation/call-budget enforcement.
- The audited path is fixture-only. It does not terminate/reap processes or establish OS isolation. Do not use it as production execution authority.
- Continue Issue #13 with worker isolation, parent watchdog/termination/reaping, independent effect verification, held-out task execution and trusted evaluation. Keep model-selected production tools disabled until those gates pass.


## Current handoff — 2026-11-06 (Issue #71 / PR #72 merged)

PR #72 is merged to `main` as `b9450142becfe8446c8dc98455977fd4a3e1fe51`. Exact-head CI run [#42227907571](https://github.com/bjo163/rocksoul/actions/runs/42227907571) passed Linux format/Clippy/tests/offline evaluations and Windows release build/tests/installer/portable smoke tests.

Read `src/worker_protocol.rs` and [the Issue #71 worker IPC plan](plans/issue-71/worker-ipc-contract.md). The merged contract is protocol-only: it does not start or supervise a process, authenticate an IPC peer, enforce OS isolation, terminate/reap a process, or enable production dispatch. Rust 1.85 is preserved; no runtime dependency or lockfile change was made.

Next safe sequence:
1. Implement parent supervisor with authenticated local IPC, bounded handshake/heartbeat, deadlines, cancellation, process termination and OS-observed reaping.
2. Implement Linux isolation and negative tests (namespaces, seccomp, cgroups v2, minimal read-only mounts, no network).
3. Implement Windows isolation and negative tests (restricted token/AppContainer policy, Job Object limits, explicit handle inheritance, kill-on-close and reaping).
4. Select a WASM runtime only after an explicit MSRV/license/security decision; no WASI and no default imports.
5. Keep the independent effect verifier in a separate trust domain; then implement held-out model task evaluation and trusted RID publication gates.

Never enable production tool dispatch from the protocol contract alone. Issue #13 remains the controlling acceptance gate.
