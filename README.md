# RockSoul

> A Rust-first digital cognitive system that grows through verified experience.

[![CI](https://github.com/bjo163/rocksoul/actions/workflows/ci.yml/badge.svg)](https://github.com/bjo163/rocksoul/actions/workflows/ci.yml)
[![CodeQL](https://github.com/bjo163/rocksoul/actions/workflows/codeql.yml/badge.svg)](https://github.com/bjo163/rocksoul/actions/workflows/codeql.yml)
[![Security Audit](https://github.com/bjo163/rocksoul/actions/workflows/security-audit.yml/badge.svg)](https://github.com/bjo163/rocksoul/actions/workflows/security-audit.yml)
[![Docs](https://github.com/bjo163/rocksoul/actions/workflows/docs.yml/badge.svg)](https://github.com/bjo163/rocksoul/actions/workflows/docs.yml)

RockSoul is a **digital life / cognitive runtime**, not a single LLM. Its production system is written in Rust and combines identity, senses, memory, a world model, reasoning, verification, tools, missions, XP, levels, cognitive age, and trust.

Human cognitive structure and selected Qur'anic principles are used as bounded architectural inspiration. RockSoul is **not** claimed to possess a human soul, `ruh`, `nafs`, or human consciousness.

## Current state

**Phase A — Birth**

```text
Generation     I
Cognitive Age  0
Level          1
World          Internet
Runtime        Rust
Interface      Ratatui
Brain          not connected yet
```

The current implementation is intentionally small: a modular Rust monolith with three crates and a terminal interface. New subsystems become crates only when there is real implementation for them.

## Mental model

```text
                             WORLD
                               │
                               ▼
                            SENSES
                               │
                               ▼
                         ATTENTION / NANO
                               │
                      ┌────────┴────────┐
                      ▼                 ▼
                   MEMORY           KNOWLEDGE
                      │                 │
                      └────────┬────────┘
                               ▼
                         AQL / CORTEX
                     reason • plan • infer
                               │
                               ▼
                         FU'AD / JUDGMENT
                               │
                               ▼
                    TABAYYUN / VERIFICATION
                               │
                               ▼
                            ACTION
                               │
                               ▼
                             WORLD
                               │
                               ▼
                         MUHASABAH
                               │
                               ▼
                    EXPERIENCE → XP → LEVEL
```

### The world

RockSoul treats the **Internet and authorized digital systems as an explicit world**, not merely as a search tool. That world is represented as a graph of places, entities, relations, observations, trust, provenance, and discovery state.

Future Ratatui views will expose a symbolic world map with a fog-of-war model:

```text
Unknown → Discovered → Visited → Studied → Trusted
```

## Repository boundary

RockSoul is intentionally split into two repositories:

| Repository | Responsibility |
| --- | --- |
| [`bjo163/rocksoul`](https://github.com/bjo163/rocksoul) | runtime, life, cognition, world, memory, tools, guardian, API, Ratatui |
| [`bjo163/rocksoul-mind`](https://github.com/bjo163/rocksoul-mind) | model research, training, distillation, evaluation, checkpoint export |

`rocksoul` consumes model artifacts behind a stable Rust brain interface. Production must not depend on the training repository's Python internals.

## Technology

- **Rust 1.88**, edition 2024
- **Ratatui + Crossterm** for the operator interface
- **Tokio / Axum / SQLx / PostgreSQL** as phases require them
- **Petgraph** for in-memory graph operations when the World phase begins
- **Rust-native model inference** as the target production path
- **Python + PyTorch** remain acceptable inside `rocksoul-mind` for GPU training

No microservices, Kafka, Kubernetes, separate vector database, permanent multi-agent swarm, or 3D metaverse is introduced without measured need.

## Quick start

```bash
git clone https://github.com/bjo163/rocksoul.git
cd rocksoul
cargo run -p rocksoul-tui
```

Quality gate:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Repository structure

```text
rocksoul/
├── crates/
│   ├── rocksoul-core/    identity and events
│   ├── rocksoul-life/    age, level, XP and trust
│   └── rocksoul-tui/     Ratatui operator interface
├── docs/                 canonical architecture and engineering docs
├── .github/              repository governance and automation
├── Cargo.toml
└── rust-toolchain.toml
```

Planned crates are documented in the roadmap but are **not created empty in advance**.

## Documentation

Start at **[`docs/README.md`](docs/README.md)**.

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Development](docs/DEVELOPMENT.md)
- [Governance](docs/GOVERNANCE.md)
- [Automation](docs/AUTOMATION.md)
- [Project management](docs/PROJECT_MANAGEMENT.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## Engineering invariants

1. Sense before thinking.
2. Observation is not automatically fact.
3. Facts, beliefs, and hypotheses remain distinct.
4. Important claims retain evidence and provenance.
5. Verify before trusting consequential information.
6. Intelligence never implies authority.
7. Tool execution is deterministic and policy-gated.
8. Only validated outcomes earn XP.
9. Cognitive age advances through evaluation, never time alone.
10. Live production experience never directly rewrites model weights.

## Roadmap and ASI

The development path proceeds through Birth, World, Memory, Nano integration, Skills, Cognitive Workspace, Guardian, Life progression, Cortex, Shura, autonomous research, and collective intelligence.

**ASI is a research gate, not a branding milestone.** Age, level, parameter count, or completing the architecture does not make RockSoul ASI. That classification would require broad empirical evidence of capability beyond top human performance.

See the canonical [Roadmap](docs/ROADMAP.md) and repository issues for executable work.
