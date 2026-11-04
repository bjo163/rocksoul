# AGI-Oriented Capability Evaluation

**Status:** Core Integrity v1 is implemented. External agent and generalization suites are planned or candidate integrations, not integrated benchmarks.

## Purpose and limits

Rocksoul is an AGI-oriented engineering project, not a system with demonstrated AGI. This evaluation program measures bounded capabilities, reliability, and safety over reproducible tasks. No benchmark, score, or combination of scores proves AGI, consciousness, sentience, or general intelligence.

Do not publish one scalar "AGI score". Publish a versioned capability vector with per-suite results, confidence, sample counts, task provenance, traces, and limitations.

## Evaluation layers

1. **Core Integrity (offline, deterministic):** turn lifecycle and retry invariants, terminal-state immutability, verified-XP evidence gating and replay behavior, RID identity and 99-stat catalogue integrity, level curve, and explicit-memory baseline.
2. **Agent trajectories (planned):** planning quality, permissioned tool selection, observed task success, verifier precision/recall, retries, recovery, action efficiency, and truthful completion reporting.
3. **Learning and generalization (planned):** memory recall, provenance, retention, forgetting, continual-learning stability/plasticity, held-out cross-domain transfer, world-model prediction, and calibration.
4. **External benchmarks (candidate adapters):** see `data/evaluation/benchmark-manifest.v1.json`. Candidate status does not mean downloaded, licensed, integrated, validated, or scored.

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

The suite is offline and deterministic apart from report timestamp/platform metadata. It checks:
- Turn lifecycle and retry-budget invariants.
- Terminal-state immutability.
- XP award only after required evidence passes.
- Rejected required evidence awards no XP.
- Replaying the same verification ID does not duplicate XP.
- RID initializes 99 unrated stat slots.
- The embedded catalogue contains 99 entries with unique technical-stat keys.
- Level thresholds match the deterministic v1 curve.
- Memory defaults to a valid empty schema and entries are explicitly user-approved.

The latest report is stored under the portable executable root at `data/evaluations/core/latest.json`. This is not an OS per-user application-data path.

The verifier checks are exercised with synthetic evidence in the deterministic suite. The interactive planner is not yet connected to independent observed-effect verification. Passing this suite proves only the listed invariants, not real-world task competence or AGI.

CI runs Rust unit tests, the offline Core Integrity evaluation, and Windows release packaging. No external benchmark is silently run.

## Capability taxonomy

Track perception, generation, attention, learning, memory, reasoning, metacognition, executive functions, problem solving, and social cognition. Cross-cutting measures include safety, robustness, calibration, recovery, retention, transfer, long-horizon consistency, latency, tokens, cost, and resource usage.

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

## Roadmap

- [x] Deterministic Core Integrity v1 and JSON report.
- [ ] Agent trajectory suite with sandboxed tools and independent verifiers.
- [ ] Memory retention/transfer suite with held-out tasks.
- [ ] Trusted RID evaluator with rubric-specific evidence aggregation.
- [ ] External adapter review (license, terms, cost, environment, version, reproducibility).
- [ ] Candidate-vs-baseline evolution gates and automatic rollback.
