# RockSoul Roadmap

GitHub milestones and canonical roadmap issues are the executable source of truth for phase work. This document defines ordering, intent, and exit criteria.

## Phase A — Birth

Goal: a minimal, buildable Rust application with an explicit, restart-safe life identity and Ratatui shell.

Completed foundation:

- [x] repository and branch baseline
- [x] Rust workspace and pinned toolchain
- [x] `rocksoul-core` identity and event primitives
- [x] `rocksoul-life` age/level/XP/trust state
- [x] validated-XP gate
- [x] cognitive-age evaluation gate
- [x] initial Ratatui HOME screen
- [x] exact-head CI green on `main`

Remaining canonical work:

- persistent birth/life state
- append-only event journal
- chronological age derived from persisted birth event
- restart/recovery tests

Exit criteria:

- `cargo run -p rocksoul-tui` opens the Home interface;
- identity survives restart without a new birth;
- lifecycle mutations are journaled and testable;
- exact-head CI remains green.

## Phase B — World

Goal: make the Internet and authorized digital environment an explicit navigable world model.

- world/place/entity primitives
- world graph
- discovery state and fog of war
- current location/activity
- Ratatui WORLD view
- provenance/trust metadata
- finite exploration budgets

No 3D renderer in this phase.

## Phase C — Memory

- PostgreSQL persistence
- working memory
- episodic memory
- semantic memory
- procedural memory
- provenance, freshness and supersession metadata
- pgvector only when semantic retrieval demonstrates a need

## Phase D — RockSoul Nano

Consume a versioned exported artifact from `bjo163/rocksoul-mind` through a stable Rust brain interface.

Initial capabilities:

- intent detection
- attention/routing
- entity extraction
- complexity classification
- structured output
- simple tool selection

Production inference must not require the training repository or a Python service.

## Phase E — Skills / Yad

Begin with read-oriented skills:

- web/search abstraction
- files
- GitHub
- HTTP/API
- PostgreSQL read

Privileged/domain tools arrive only behind explicit authorization and risk policy.

## Phase F — Cognitive Workspace

Canonical structured task state:

- goals
- constraints
- facts
- beliefs
- unknowns
- hypotheses
- evidence
- plans
- actions
- observations
- confidence
- budgets

## Phase G — Tabayyun / Guardian

- provenance validation
- schema validation
- freshness
- contradiction checks
- permission policy
- action risk classes
- audit trail

## Phase H — Life Progression

- mission model
- validated XP formula
- domain XP
- level progression
- skill tree
- achievements
- cognitive-age benchmark gates
- trust/delegation metrics

## Phase I — Cortex

Research target: approximately 300–600M parameters, subject to evidence from `rocksoul-mind`.

- reasoning
- planning
- hypotheses
- research synthesis
- world-state reasoning
- consequence simulation

## Phase J — Shura

Add dynamic multi-perspective reasoning only when task complexity warrants it. Avoid permanent agent swarms.

## Phase K — Autonomous Research

Goal → research → hypothesis → experiment/code → observe → verify → report.

All loops remain finite, policy-gated, observable, and auditable.

## Phase L — Collective Intelligence

Controlled sharing of verified world knowledge and validated experience between RockSoul instances.

## Phase M — ASI Research Gate

ASI is not unlocked by level, age, architecture, parameter count, or branding. It remains a research classification requiring broad empirical evidence of capabilities beyond top human performance.
