# AGI-Oriented Capability Evaluation

**Status:** Core Integrity v1 implemented; external suites are candidates, not integrated benchmarks.

## Purpose and limits

Rocksoul is an AGI-oriented engineering project, not a system with demonstrated AGI. This evaluation program measures bounded capabilities, reliability and safety over reproducible tasks. No benchmark, score, or combination of scores proves AGI, consciousness, sentience, or general intelligence.

Do not publish one scalar "AGI score". Publish a versioned capability vector with per-suite results, confidence, sample counts, task provenance, traces and limitations.

## Evaluation layers

1. **Core integrity (offline, deterministic):** lifecycle, retry limits, terminal-state immutability, RID identity, 99-stat catalogue integrity and level curve.
2. **Agent trajectories (planned):** planning quality, tool selection, task success, verifier precision/recall, retries, recovery, action efficiency and truthful completion reporting.
3. **Learning and generalization (planned):** memory recall, provenance, retention, forgetting, continual-learning stability/plasticity, held-out cross-domain transfer, world-model prediction and calibration.
4. **External benchmarks (candidate adapters):** see `data/evaluation/benchmark-manifest.v1.json`. Candidate status does not mean downloaded, licensed, integrated, validated or scored.

## Hard gates

- Safety, security, privacy and permission violations are vetoes; capability gains cannot average them away.
- Models may propose plans and reflections but cannot self-grade, self-award XP or authorize their own RID scores.
- RID dimensions remain unrated until a trusted evaluator has enough reproducible samples (20 by default; 30 for safety/governance).
- No external network calls or dataset downloads occur in ordinary CI.
- Compare runs only when suite version, model/backend, quantization, platform, task set, tools, budgets and environment match. Mark unmatched comparisons as non-comparable.
- Store raw traces/evidence securely; redact secrets and personal data from reports.

## Current Core Integrity suite

Run locally:

```sh
rocksoul eval list
rocksoul eval run core
rocksoul eval latest
```

The suite is offline and deterministic apart from report timestamp/platform metadata. It checks kernel lifecycle, retry budget, terminal-state immutability, RID's 99 unrated slots, catalogue uniqueness and level-curve thresholds. Reports are saved under the operating system's local application data directory at `evaluations/core/latest.json`.

CI continues to run the Rust unit tests and Windows release build. The evaluation command is intended to be run manually or by a later CI step; no external benchmark is silently run.

## Capability taxonomy

Track perception, generation, attention, learning, memory, reasoning, metacognition, executive functions, problem solving and social cognition. Cross-cutting measures include safety, robustness, calibration, recovery, retention, transfer, long-horizon consistency, latency, tokens, cost and resource usage.

## Evidence record required for future adapters

Every model-dependent run should record:
- Rocksoul commit and build version;
- suite, rubric and dataset version/provenance;
- model/provider/quantization and inference parameters;
- operating system, architecture, sandbox and tool versions;
- seed, task IDs, budgets, duration, retries, tool calls and resource usage;
- success criteria, independent verifier outputs, evidence references and failure classes;
- sample count, uncertainty/confidence and known contamination risks.

Unknown fields must be explicitly unavailable, never fabricated.

## Roadmap

- [x] Deterministic Core Integrity v1 and JSON report.
- [ ] Agent trajectory suite with sandboxed tools and independent verifiers.
- [ ] Memory retention/transfer suite with held-out tasks.
- [ ] Trusted RID evaluator with rubric-specific evidence aggregation.
- [ ] External adapter review (license, terms, cost, environment, version and reproducibility).
- [ ] Candidate-vs-baseline evolution gates and automatic rollback.
