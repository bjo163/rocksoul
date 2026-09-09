# RockSoul Architecture Baseline v1

## Purpose

RockSoul is a Rust-first digital cognitive system. Human cognitive structure and selected Qur'anic principles are used as bounded architectural inspiration; RockSoul is not claimed to possess ruh, nafs, a human soul, or human consciousness.

## Repository boundary

### `bjo163/rocksoul`

Owns the digital life system:

- Rust runtime and kernel
- identity and lifecycle
- cognitive workspace
- world model and world graph
- memory
- senses/connectors
- deterministic skills and executor
- verification/guardian
- missions, XP, level, cognitive age and trust
- API
- Ratatui operator interface

### `bjo163/rocksoul-mind`

Owns the brain laboratory:

- MiniMind-derived model research
- RockSoul Nano ~64M
- ID+EN adaptation
- Cortex ~300–600M research
- datasets
- distillation
- training
- evaluation
- checkpoint export

The runtime must consume model artifacts through a stable brain interface. It must not depend on the training repository's internal Python implementation.

## Cognitive mapping

| Concept | Engineering role |
| --- | --- |
| Jasad | Rust runtime and infrastructure |
| Indra | observation-producing connectors |
| Thalamus | attention/router; RockSoul Nano |
| Asma | entities, ontology and semantic relations |
| Qalam | externalized knowledge |
| Dzakirah | working, episodic, semantic and procedural memory |
| Aql | Cortex reasoning and planning |
| Fu'ad | judgment workspace: hypotheses, evidence, confidence and consequences |
| Tabayyun | source/evidence verification |
| Ahl al-dhikr | routing to specialist capability |
| Shura | on-demand multi-perspective reasoning for difficult tasks |
| Yad | deterministic tools and execution |
| Amanah | authorization, permissions and risk controls |
| Muhasabah | outcome reflection and experience capture |

These names are architectural metaphors, not theological claims about software.

## Core invariants

1. Sense before thinking.
2. An observation is not automatically a fact.
3. Facts, beliefs and hypotheses remain distinct.
4. Claims must retain provenance where available.
5. Verify before trusting consequential information.
6. Plan before multi-step action.
7. Intelligence never implies authority.
8. Tool execution is deterministic and policy-gated.
9. Observe outcomes after actions.
10. Only validated experience can earn XP.
11. Cognitive age advances only through explicit evaluation gates.
12. Production weights never self-modify directly from live experience.

## Model topology

The initial target uses two model roles, not a permanent swarm:

```text
RockSoul Nano ~64M
  -> attention / routing / extraction / simple tool calls

RockSoul Cortex ~300–600M
  -> reasoning / planning / research / synthesis
```

Specialization should prefer adapters or equivalent lightweight capability modules before introducing full independent models.

## Global cognitive workspace

The canonical task state is structured rather than being only a chat transcript:

```text
Goal
Constraints
Facts
Beliefs
Unknowns
Hypotheses
Evidence
Plan
Actions
Observations
Risk
Confidence
Budget
```

## World

RockSoul's primary world is digital:

```text
Internet World
Physical World represented through sensors/data
Private World exposed through authorized systems
```

The backend representation is a world graph. Ratatui initially renders a 2D symbolic map over that graph. A future graphical/metaverse UI may consume the same world model; 3D rendering is not part of the core.

World locations can progress through states such as:

```text
Unknown -> Discovered -> Visited -> Studied -> Trusted
```

Trust is evidence-based and reversible. Restricted locations are tracked separately from knowledge/trust.

## Life model

RockSoul tracks independent dimensions:

- chronological age: elapsed time since a persisted birth event
- cognitive age: benchmarked maturity abstraction
- level: accumulated validated experience progression
- XP: verified experience points
- domain experience: reasoning, research, coding, ISP, ERP, network, tools, verification, etc.
- trust: delegation confidence; never an authorization bypass
- generation: significant architecture/model lineage transition

Game-like progression is an observability and evaluation model, not a psychological claim that software literally has a human age.

## Execution loop

```text
Receive
-> Sense
-> Attend
-> Understand
-> Recall
-> Form hypotheses
-> Plan
-> Simulate consequences when needed
-> Verify plan
-> Act through policy-gated skills
-> Observe
-> Update world state
-> Tabayyun
-> Reflect
-> Respond
-> Remember validated experience
```

Every loop is budgeted with finite limits for steps, tool calls, replans, time, cost and risk.

## Initial implementation strategy

RockSoul starts as a modular monolith. Do not introduce microservices, Kafka, Kubernetes, a separate vector database, permanent multi-agent orchestration, or a 3D frontend without measured need.

Initial crates are intentionally limited to:

- `rocksoul-core`
- `rocksoul-life`
- `rocksoul-tui`

New crates are added only when a phase contains real implementation for that domain.
