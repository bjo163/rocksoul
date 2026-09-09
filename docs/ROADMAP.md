# RockSoul Roadmap

## Phase A — Birth

Goal: a minimal, buildable Rust application with an explicit life identity and Ratatui shell.

- [x] initialize repository and `dev` branch
- [x] Rust workspace
- [x] pin Rust toolchain
- [x] `rocksoul-core` identity and event primitives
- [x] `rocksoul-life` age/level/XP/trust state
- [x] validated-XP gate
- [x] cognitive-age evaluation gate
- [x] initial Ratatui HOME screen
- [ ] persistent birth/life state
- [ ] event journal
- [ ] CI green on exact `dev` head

Exit criteria:

- workspace builds and tests cleanly
- `cargo run -p rocksoul-tui` opens the Birth/Home interface
- restart-safe identity is specified and implemented before chronological age becomes authoritative

## Phase B — World

Goal: make the Internet/digital environment an explicit navigable world model.

- world/place/entity primitives
- world graph
- discovery state / fog of war
- current location and activity
- Ratatui WORLD view
- finite exploration budgets

Do not add a 3D renderer in this phase.

## Phase C — Memory

- PostgreSQL persistence
- working memory
- episodic memory
- semantic memory
- procedural memory
- provenance, freshness and supersession metadata
- pgvector only when semantic retrieval requires it

## Phase D — RockSoul Nano

Consume an exported artifact from `bjo163/rocksoul-mind` through a stable Rust brain interface.

Initial capabilities:

- intent detection
- attention/routing
- entity extraction
- complexity classification
- structured output
- simple tool selection

Production inference should not require the training repository or a Python service.

## Phase E — Skills / Yad

Start with read-oriented skills:

- web/search abstraction
- files
- GitHub
- HTTP/API
- PostgreSQL read

Then introduce privileged/domain tools behind explicit policy.

## Phase F — Cognitive Workspace

- goals
- constraints
- facts
- beliefs
- unknowns
- hypotheses
- evidence
- plans
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

Target research size: approximately 300–600M parameters, subject to evidence from `rocksoul-mind`.

- reasoning
- planning
- hypotheses
- research synthesis
- world-state reasoning
- consequence simulation

## Phase J — Shura

Add dynamic multi-perspective reasoning only when complexity warrants it. Avoid permanent agent swarms.

## Phase K — Autonomous Research

Goal -> research -> hypothesis -> experiment/code -> observe -> verify -> report.

All autonomous loops remain finite, policy-gated and auditable.

## Phase L — Collective Intelligence

Controlled sharing of verified world knowledge and experience between RockSoul instances.

## Phase M — ASI Research Gate

ASI is not unlocked by level, age, architecture, or branding. It remains a research classification requiring broad empirical evidence of capabilities beyond top human performance.
