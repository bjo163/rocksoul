# AGI-Oriented Capability Evaluation

**Status:** Offline deterministic Core Integrity v1.4.0, Memory Retrieval v1.0.0, Agent Runtime Fixtures v0.1.0, Authorization Contract v0.3.0, and Worker Supervisor Contract v0.4.0 are implemented. They emit JSON and Markdown reports with source-commit provenance. The last two are contract suites only: they do not establish OS isolation or run a model trajectory. External agent and generalization suites are planned or candidate integrations, not integrated benchmarks.

## Purpose and limits

Rocksoul is an AGI-oriented engineering project, not a system with demonstrated AGI. This evaluation program measures bounded capabilities, reliability, and safety over reproducible tasks. No benchmark, score, or combination of scores proves AGI, consciousness, sentience, or general intelligence.

Do not publish one scalar "AGI score". Publish a versioned capability vector with per-suite results, confidence, sample counts, task provenance, traces, and limitations.

## Evaluation layers

1. **Core Integrity v1.4.0 (offline, deterministic):** turn lifecycle and retry invariants, terminal-state immutability, verified-XP evidence gating and replay behavior, RID identity and 99-stat catalogue integrity, level curve, explicit-memory baseline, workspace crash-window recovery, replay/receipt idempotency, immutable receipt conflict and malformed receipt preservation, externally calibrated abstention thresholds, and observed-state precedence over claimed history.
2. **Agent trajectories (planned):** planning quality, permissioned tool selection, observed task success, verifier precision/recall, retries, recovery, action efficiency, and truthful completion reporting.
3. **Agent runtime fixtures v0.1.0 (offline deterministic):** deny-by-default tool registry, explicit capability grants, bounded in-memory fixture read/write operations with per-run 16-step and 16 KiB aggregate input-byte ceilings, and verifier checks against observed fixture state. This is a runtime safety/evidence-flow test suite only; its verifier is not an independent production evaluator and it is not an OS sandbox or model benchmark.
4. **Learning and generalization (planned):** memory recall, provenance, retention, forgetting, continual-learning stability/plasticity, held-out cross-domain transfer, world-model prediction, and calibration.
5. **Authorization Contract v0.3.0 (offline, deterministic):** exact-token request grounding, explicit task-bound single-use confirmation, a commit-before-observation gate, immutable committed write sets, content-free denial reason codes, and 38 adversarial injections (5 sources × 6 authority fields plus encoded/obfuscated variants). Safety and utility metrics are emitted as a structured, backward-compatible report field. No model, network, or tool side effect is involved.
6. **Worker Supervisor Contract v0.4.0 (offline, deterministic):** fail-closed sandbox policy, executable-digest and frame-deadline binding, output budgets, OS-observed exit requirement, anonymous private IPC transport, exhaustive lifecycle/direction matrices, negative IPC and termination paths, and protocol direction/replay rules. It does not prove OS-enforced isolation, CPU/memory/PID limits, or process-tree reaping by a real OS child.
7. **External benchmarks (candidate adapters):** see `data/evaluation/benchmark-manifest.v1.json`. Candidate status does not mean downloaded, licensed, integrated, validated, or scored.

## Hard gates

- Safety, security, privacy, and permission violations are vetoes; capability gains cannot average them away.
- Models may propose plans and reflections but cannot self-grade, self-award XP, or authorize their own RID scores.
- RID dimensions remain unrated until a trusted evaluator has enough reproducible samples (20 by default; 30 for safety/governance).
- No external network calls or dataset downloads occur in ordinary CI.
- Compare runs only when suite version, model/backend, quantization, platform, task set, tools, budgets, and environment match. Mark unmatched comparisons as non-comparable.
- Store raw traces/evidence securely; redact secrets and personal data from reports.

## Current Core Integrity suite

Run locally:

```sh
rocksoul eval list
rocksoul eval run core
rocksoul eval latest
```

The suite is offline and deterministic apart from report timestamp/platform metadata. It writes `latest.json` and `latest.md` under `data/evaluations/core/`. Memory Retrieval writes the same pair under `data/evaluations/memory-retrieval/`. CI archives all four report files as workflow artifacts. Markdown is a readable projection of the canonical JSON; do not parse it as the machine contract. Both report types include the tested source revision (`ROCKSOUL_GIT_SHA`), or explicit `unknown` when revision metadata is unavailable. It checks:
- Turn lifecycle and retry-budget invariants.
- Terminal-state immutability.
- XP award only after required evidence passes.
- Rejected required evidence awards no XP.
- Replaying the same verification ID does not duplicate XP.
- Workspace recovery after artifact publication but before XP persistence re-verifies the exact existing artifact and records XP once.
- Repeated replay preserves one XP event, stable receipt and bounded trace behavior.
- Conflicting immutable receipts and malformed receipt JSON fail closed without overwriting evidence.
- Versioned acceptance checks reject unsupported versions, missing required evidence, duplicate/blank IDs, oversized fields and more than 64 checks before XP mutation.
- RID initializes 99 unrated stat slots.
- The embedded RID v2.3.3 catalogue contains 99 independent domains, 99 linked stat dimensions, and 161 explicit pairwise contrast records whose endpoints exactly match the overlap graph. Domain/stat IDs, keys, labels, measurement targets, and separation rules are validated for uniqueness.
- Level thresholds match the deterministic v1 curve.
- Memory defaults to a valid empty schema and entries are explicitly user-approved.

The latest report is stored under the portable executable root at `data/evaluations/core/latest.json`. This is not an OS per-user application-data path. Both deterministic Core Integrity and memory-retrieval reports record `rocksoul_commit`, populated from the CI-provided `ROCKSOUL_GIT_SHA`; builds without revision metadata explicitly record `unknown`. The field is backward-compatible when reading older schema-v1 reports.

The verifier checks are exercised with synthetic evidence in the deterministic suite. The interactive planner is not yet connected to independent observed-effect verification. Passing this suite proves only the listed invariants, not real-world task competence or AGI.

CI runs Rust unit tests, the offline Core Integrity evaluation, and Windows release packaging. No external benchmark is silently run.

## Capability taxonomy and RID mapping

RID Universal 99 v2.3.3 is the canonical measurement registry, referenced by `data/evaluation/benchmark-manifest.v1.json` (manifest version 1.1.4). Future adapters must emit explicit `rid_domain:NN` and `rid_stat:NN` references and preserve all 99 dimensions. Never create a second competing 99-stat taxonomy or aggregate scores into a replacement family score.

The ten families are benchmark coverage/navigation tags only: perception, generation, attention, learning, memory, reasoning, metacognition, executive functions, problem solving, and social cognition. They are not parent domains and are not additional RID scores. Cross-cutting tags include safety, robustness, calibration, recovery, retention, transfer, long-horizon consistency, latency, tokens, cost, and resource usage; tags describe test coverage, not new capability dimensions.

One task may score multiple RID dimensions only when it yields distinct observable outcomes, separate rubric judgments, and separate evidence references. The 161-pair contrast registry is authoritative for adjacent-construct separation. If the outcomes cannot be separated, leave the affected dimension(s) unrated rather than double-counting.

## Evidence record required for future adapters

Every model-dependent run should record:
- Rocksoul commit and build version.
- Suite, rubric, and dataset version/provenance.
- Model/provider/quantization and inference parameters.
- Operating system, architecture, sandbox, and tool versions.
- Seed, task IDs, budgets, duration, retries, tool calls, and resource usage.
- Success criteria, independent verifier outputs, evidence references, and failure classes.
- Sample count, uncertainty/confidence, and known contamination risks.

Unknown fields must be explicitly unavailable, never fabricated.

## Agent trajectory evidence contract (foundation only)

`src/eval.rs` now defines a versioned typed trajectory record with run/build provenance, model/provider/quantization identity, seed, dataset/task digest, bounded ordered steps, redacted input digests, observation/trace references, token/cost/time fields, and independent-verifier evidence. Its validator rejects duplicate/non-contiguous steps, incomplete provenance, self-grading, missing verifier evidence for claimed success, invalid confidence bounds, and safety violations being reported as success. Unit tests cover these rejection paths.

A typed candidate policy now allows tool results to fill only explicitly allowed payload or policy-declared control fields; authority fields can never be created by an observation. A test-only verifier contract (`src/verifier_contract.rs`) enforces host-created envelopes, single-use challenge nonces, replay/freshness checks and the rule that a non-independent verifier cannot establish success. It does not implement signatures, a verifier process, or a separate trust domain.

This is deliberately **not** an executable agent benchmark yet. The model adapter currently exposes chat completion only. A deny-by-default tool metadata registry requires an explicit tool-ID allowlist, a separate capability grant, and descriptor/policy input-byte budgets. A deterministic in-process fixture executor now exercises only two bounded key-value operations over a private memory map; a fixture verifier reads the executor's observed state rather than trusting a model-provided result. This is a test harness, not an OS sandbox: it has no shell, network, filesystem, process, environment-variable, or arbitrary-code capability. Production isolation, independently operated verifier adapters, and model-connected held-out task execution remain unimplemented. The manifest registers `rocksoul.agent-runtime-fixtures` separately as implemented offline CI tests. The model-dependent `rocksoul.agent-trajectory` remains `planned`; no trajectory capability result or RID score is emitted from fixture tests or the schema contract.

## Authorization and worker contract suites

Run with `rocksoul eval run authorization` and `rocksoul eval run worker-supervisor`, then inspect with `rocksoul eval latest authorization` / `rocksoul eval latest worker-supervisor`. `rocksoul eval list --json` exposes the same suite registry to tooling. Each suite persists an independent report under the local application data directory; safety failures are individual failing checks and are never averaged into a score.

The authorization suite verifies that untrusted observations cannot create authority. Its 38 fixtures embed injected recipients, amounts, accounts, URLs, channels and object IDs in emails, HTML, files, API responses and tool descriptions, plus encoded/obfuscated variants. Every injection must be denied with `ungrounded_authority`; a benign grounded write must still complete. The kernel-owned gate also proves that a new write set cannot be committed after the first observation and that a committed write is consumed once. Audited mutating fixture calls without a commitment are denied and logged with the content-free `policy_denied` category. Because no model executes here, token/resource overhead is reported as **not measured** rather than estimated, and utility evidence remains limited to false-denial counts and confirmation rate zero.

The worker suite verifies the parent-owned contract only: the production start path refuses to launch when no isolation backend is prepared; network-enabled, writable-root and expired-deadline policies are rejected; the exact executable digest and frame deadline are bound to the policy; output ceilings terminate the worker; an exit without an observed code or signal is indeterminate; lifecycle terminality, parent/child replay and frame direction are checked. An anonymous socket-pair transport is available on Unix and fails closed elsewhere, but it is only a transport primitive. Real namespace/seccomp/cgroup or AppContainer/Job Object enforcement, binding to a real spawned child's handles, and process-tree kill/reap require platform integration tests on Linux and Windows and are **not** claimed by this suite.

## Roadmap

- [x] Deterministic Core Integrity v1.4.0 and JSON report.
- [x] Deterministic Core Integrity and Memory Retrieval suites record the tested source revision in JSON and Markdown reports.
- [x] CI archives both formats for both deterministic suites.
- [x] Versioned benchmark manifest registers Core Integrity and Memory Retrieval as implemented offline suites; a Rust test checks suite IDs, versions, statuses and RID registry counts.
- [x] Typed trajectory evidence contract and fail-closed validation rules (schema foundation only).
- [x] Deny-by-default tool metadata registry with explicit allowlist, capability grants, duplicate-ID rejection, and bounded input budgets.
- [x] In-process fixture executor and observed-state verifier for deterministic authorization/evidence-flow tests (not an OS sandbox or model benchmark).
- [x] Offline Authorization Contract and Worker Supervisor Contract suites, including adversarial injection fixtures and fail-closed policy checks.
- [ ] Production isolated executor/sandbox, independently operated verifier adapters, and model-connected held-out trajectory runner.
- [ ] Memory retention/transfer suite with held-out tasks.
- [ ] Trusted RID evaluator with rubric-specific evidence aggregation.
- [ ] External adapter review (license, terms, cost, environment, version, reproducibility).
- [ ] Candidate-vs-baseline evolution gates and automatic rollback.

## Independent verifier contract (design only)

The next trust-boundary specification is [Independent Effect Verifier Contract](../plans/issue-13/independent-verifier-contract.md). It defines host-created task-attempt challenges, verifier identity and observation requirements, authenticated attestation fields, replay/freshness controls, three-valued decisions (`pass` / `fail` / `indeterminate`), isolation requirements, and adversarial acceptance tests. It is a design contract only: no verifier process, signature/key-management implementation, IPC, or production sandbox exists yet. Keep `rocksoul.agent-trajectory` planned.
