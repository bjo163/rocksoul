# RockSoul Cognitive Runtime TODO

Status: IN_PROGRESS — CR-001..CR-011 implemented as a bounded MVP; CR-012 certification is pending the security-tool gate.  
Baseline: `main` at `af7d52366fd6beef80963967a562c81aa54b1a6e`  
Scope: cognitive runtime only; model research remains in `rocksoul-mind` and ecosystem orchestration remains in `runtime`.

## Rules for every task

- Never promote an observation, fixture, inferred value, or legacy value to canonical fact without explicit evidence and ownership.
- Preserve the separation between runtime mechanics and research semantics. Domain research remains owned by MFTL, LEGEND, SUPERHERO, RGBL, AWS, JIZZ, and CORRELATION.
- Keep the runtime provider-neutral. GitHub, GitLab, model providers, databases, and cloud storage are adapters, not semantic authorities.
- Default all new connectors and tools to read-only, bounded, auditable operation.
- A task is complete only when its acceptance criteria, tests, documentation, and concrete evidence are recorded in `MASTER-TODO.json`.

## Dependency flow

```text
CR-001 → CR-002 → CR-003 → CR-004 → CR-005 → CR-006 → CR-007
                                      └──────────────→ CR-008 → CR-009
CR-004 ───────────────────────────────────────────────→ CR-010
CR-007 + CR-008 + CR-009 + CR-010 → CR-011 → CR-012
```

## Tasks

### CR-001 — Reproducible baseline and documentation alignment

Dependencies: none  
Priority: P0  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Commit `Cargo.lock` if the application/workspace policy requires it.
- Correct stale clone, branch, deployment, and repository-status statements in the docs.
- Freeze the current public runtime baseline and record commit, toolchain, test command, and artifact hashes.
- Add a short compatibility policy for the initial schema and brain interface.

Acceptance:

- Clean baseline can be built from a fresh clone with the documented commands.
- Documentation describes the actual repository, branch, and clone state.
- No secret, machine-specific semantic path, or provider-specific assumption is introduced.

Evidence: `cargo test`, `cargo fmt --check`, `cargo clippy`, clean-tree output, baseline SHA, and documentation diff.

### CR-002 — Birth identity and append-only lifecycle journal

Dependencies: CR-001  
Priority: P0  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Define stable runtime identity, birth timestamp, schema version, and lifecycle event IDs.
- Persist append-only events for birth, start, pause, resume, stop, error, and recovery.
- Implement deterministic replay and recovery without fabricating state from corrupt records.
- Keep unknown event versions diagnosable and non-authoritative.

Acceptance:

- A fresh runtime receives one stable identity.
- Append, reload, replay, and restart produce the same state.
- Corrupt or duplicated records do not silently mutate canonical state.
- Journal writes contain no secrets or uncontrolled absolute paths.

Evidence: round-trip tests, idempotent replay tests, corruption fixtures, schema compatibility fixture, and journal inspection report.

### CR-003 — Cognitive state and evidence contracts

Dependencies: CR-002  
Priority: P0  
Owner: `rocksoul-cognitive-runtime`; semantics consumed from research owners

Actions:

- Define versioned types for `Observation`, `Evidence`, `Fact`, `Belief`, `Hypothesis`, `Unknown`, `Confidence`, and `Provenance`.
- Encode legal state transitions and prevent observation-to-fact promotion by default.
- Require source, timestamp, scope, confidence, and verification status where applicable.
- Provide JSON fixtures and backward-compatible serialization tests.

Acceptance:

- Every accepted cognitive item has provenance and an explicit epistemic status.
- `UNKNOWN`, `OBSERVED`, `INFERRED`, `VERIFIED`, and `TRUSTED` remain distinguishable.
- Invalid transitions are rejected with actionable diagnostics.
- Runtime contains no canonical domain research data.

Evidence: schema files, transition matrix, positive/negative fixtures, validator output, and contract tests.

### CR-004 — Minimal Guardian and policy envelope

Dependencies: CR-003  
Priority: P0  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Define policy decisions for read-only, simulated, reversible, and privileged actions.
- Add finite budgets for steps, tools, replans, time, cost, and risk.
- Require explicit authorization context for every action request.
- Record allow, deny, defer, and approval-required decisions in the journal.

Acceptance:

- No tool can execute without a policy decision.
- Default mode is read-only.
- Budget exhaustion stops execution deterministically.
- Policy decisions are auditable and replayable.

Evidence: policy matrix, denial tests, budget tests, audit fixtures, and smoke test proving an unapproved action cannot execute.

### CR-005 — Read-only Sense connector interface

Dependencies: CR-004  
Priority: P0  
Owner: `rocksoul-cognitive-runtime`; adapters owned by their integration repositories

Actions:

- Define a provider-neutral `SenseConnector` interface that emits observations only.
- Include connector identity, source, timestamp, freshness, scope, payload hash, and failure state.
- Implement one local deterministic fixture connector before internet or filesystem connectors.
- Add rate, size, timeout, and retry limits.

Acceptance:

- Connector output enters the evidence pipeline as observation, never directly as fact.
- Offline, stale, malformed, and partial responses are explicit states.
- Connector failures do not corrupt the world or memory state.
- No credentials are stored in source, fixtures, or journal records.

Evidence: interface/schema, fixture connector, malformed/stale/offline tests, redaction test, and sample observation report.

### CR-006 — World Graph and fog-of-war state machine

Dependencies: CR-005  
Priority: P1  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Implement typed nodes and edges with provenance and observation history.
- Import `public/world.json` as topology/seed data, not as automatically trusted truth.
- Implement monotonic, evidence-gated progression from `UNKNOWN` through `DISCOVERED`, `OBSERVED`, `VISITED`, `STUDIED`, `VERIFIED`, and `TRUSTED`.
- Preserve contradictory observations instead of overwriting them silently.

Acceptance:

- World state can be rebuilt from seed plus journal.
- State promotion requires an explicit rule and evidence.
- Duplicate and conflicting edges remain diagnosable.
- TUI displays state and provenance without implying unsupported certainty.

Evidence: graph schema, transition tests, conflict fixture, rebuild test, and TUI snapshot/manual inspection.

### CR-007 — Local-first memory and recall

Dependencies: CR-006  
Priority: P1  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Separate episodic, semantic, procedural, and provenance records.
- Store only validated or explicitly marked candidate experience.
- Implement deterministic recall with scope, freshness, confidence, and source filters.
- Provide retention, redaction, and corruption handling policies.

Acceptance:

- Memory survives restart and can be rebuilt from append-only records.
- Unverified information is visibly marked during recall.
- Recall never silently becomes canonical domain knowledge.
- Redaction and retention behavior are tested.

Evidence: persistence tests, recall ranking fixtures, provenance tests, redaction tests, and recovery report.

### CR-008 — RockSoul Nano/MiniMind brain adapter

Dependencies: CR-007  
Priority: P1  
Owner: `rocksoul-cognitive-runtime` adapter; training remains `rocksoul-mind`

Actions:

- Define versioned brain interface for attention, routing, extraction, and bounded tool-intent proposals.
- Consume exported artifacts from `rocksoul-mind`; do not embed training code in the runtime.
- Validate model metadata, input/output schema, context limits, and artifact hash.
- Keep model output advisory until verified and authorized by the runtime.

Acceptance:

- Runtime can run with no model installed using deterministic fallback behavior.
- Incompatible or tampered artifacts are rejected.
- Nano output cannot directly execute a privileged action.
- Adapter tests use a deterministic fixture artifact.

Evidence: brain contract, fixture artifact metadata, hash validation test, fallback test, compatibility test, and adapter smoke output.

### CR-009 — Cognitive Workspace and execution loop

Dependencies: CR-008  
Priority: P1  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Implement workspace fields: Goal, Constraints, Facts, Beliefs, Unknowns, Hypotheses, Evidence, Plan, Actions, Observations, Risk, Confidence, and Budget.
- Implement bounded loop: Receive → Sense → Attend → Understand → Recall → Hypothesize → Plan → Verify → Act → Observe → Reflect → Respond.
- Make every transition journaled and replayable.
- Separate proposal from approval and execution.

Acceptance:

- A deterministic fixture completes the full loop without external side effects.
- Missing evidence blocks or defers action rather than inventing certainty.
- Replanning is budgeted and observable.
- Workspace state can be inspected through a stable machine-readable snapshot.

Evidence: state-machine tests, end-to-end fixture transcript, budget exhaustion test, replay test, and snapshot contract.

### CR-010 — Deterministic skills and action sandbox

Dependencies: CR-004  
Priority: P1  
Owner: `rocksoul-cognitive-runtime`; domain adapters remain separate

Actions:

- Define skill manifest with capability, input schema, side effects, reversibility, permissions, and budgets.
- Implement simulation/dry-run mode before real execution.
- Add one harmless local read-only skill and one simulated reversible skill.
- Add outcome observation and failure recovery.

Acceptance:

- Skills are deny-by-default and policy-gated.
- Simulation never mutates external state.
- Real side effects are explicitly labeled and auditable.
- Skill output becomes observation/evidence, not automatic truth.

Evidence: manifests, policy tests, dry-run transcript, side-effect guard test, and outcome journal inspection.

### CR-011 — Cortex reasoning and multi-perspective review

Dependencies: CR-007, CR-008, CR-009, CR-010  
Priority: P2  
Owner: `rocksoul-cognitive-runtime` adapter; research/training remains external

Actions:

- Define Cortex interface for reasoning, planning, synthesis, and uncertainty reporting.
- Require plans to reference evidence, constraints, risk, and expected outcomes.
- Add optional Shura-style review using independent perspectives without silently changing ownership.
- Add simulation-before-action for multi-step plans.

Acceptance:

- Cortex proposes plans but cannot bypass Guardian.
- Unsupported claims remain hypotheses or unknowns.
- Multi-perspective output preserves disagreement and provenance.
- Plan simulation detects bounded classes of failure before execution.

Evidence: plan schema, adversarial fixtures, disagreement fixture, simulation tests, and policy integration test.

### CR-012 — Integrated certification and release gate

Dependencies: CR-011  
Priority: P0  
Owner: `rocksoul-cognitive-runtime`

Actions:

- Run full format, build, test, clippy, security, schema, replay, policy, and smoke gates.
- Verify clean working tree, artifact hashes, documentation synchronization, and no secret leakage.
- Verify deterministic fallback, read-only connector behavior, action denial, journal replay, and model artifact validation.
- Record certification evidence in `MASTER-TODO.json` before any release or promotion.

Acceptance:

- All required gates pass on the declared baseline.
- No unresolved P0 safety or semantic-boundary issue remains.
- Release artifact is reproducible and its hash is recorded.
- Runtime, docs, contracts, registry, and TODO all describe the same behavior.

Evidence: CI run, local verification transcript, artifact hashes, security scan, replay report, policy report, and certification record.

## Explicit non-goals

- Do not merge this repository with `runtime`, `rocksoul-crayon`, or research-owner repositories.
- Do not claim AGI, consciousness, ruh, nafs, or biological senses from the architecture alone.
- Do not train models inside this runtime repository.
- Do not enable unrestricted internet, filesystem mutation, deployment, or self-modifying model weights.
- Do not begin GitHub-to-GitLab migration from this roadmap.

## Next work after CR-012

`CR-013` is tracked as [GitHub issue #32](https://github.com/bjo163/rocksoul/issues/32).

Implement the persistent runtime facade and operator surface: restore LifeState, journal, World, Memory, Guardian, and Workspace after restart; expose one versioned JSON snapshot to CLI/TUI; and verify the read-only/simulated vertical slice through the operator interface.
