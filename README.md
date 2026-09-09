<div align="center">

# RockSoul

**Local-first Rust digital cognitive runtime — one life, one state contract, synchronized TUI + Web surfaces.**

[![Gate](https://img.shields.io/github/actions/workflow/status/bjo163/rocksoul/gate.yml?branch=main&label=gate&logo=githubactions)](https://github.com/bjo163/rocksoul/actions/workflows/gate.yml)
[![Release](https://img.shields.io/github/v/release/bjo163/rocksoul?display_name=tag&sort=semver&label=age%20release)](https://github.com/bjo163/rocksoul/releases)
[![Downloads](https://img.shields.io/github/downloads/bjo163/rocksoul/total?label=downloads)](https://github.com/bjo163/rocksoul/releases)
[![Issues](https://img.shields.io/github/issues/bjo163/rocksoul?label=issues)](https://github.com/bjo163/rocksoul/issues)
[![Stars](https://img.shields.io/github/stars/bjo163/rocksoul?style=flat&label=stars)](https://github.com/bjo163/rocksoul/stargazers)
[![Forks](https://img.shields.io/github/forks/bjo163/rocksoul?style=flat&label=forks)](https://github.com/bjo163/rocksoul/forks)
[![Last Commit](https://img.shields.io/github/last-commit/bjo163/rocksoul?label=last%20commit)](https://github.com/bjo163/rocksoul/commits/main)
[![Rust](https://img.shields.io/badge/Rust-1.88-000000?logo=rust)](rust-toolchain.toml)
[![Local First](https://img.shields.io/badge/architecture-local--first-0e8a16)](docs/STORAGE.md)
[![Template Ready](https://img.shields.io/badge/repository-template--ready-5319e7)](docs/TEMPLATE.md)

[**Wiki**](https://github.com/bjo163/rocksoul/wiki) ·
[**Issues**](https://github.com/bjo163/rocksoul/issues) ·
[**Discussions**](https://github.com/bjo163/rocksoul/discussions) ·
[**Projects**](https://github.com/users/bjo163/projects?query=rocksoul) ·
[**Actions**](https://github.com/bjo163/rocksoul/actions) ·
[**Releases / EXE**](https://github.com/bjo163/rocksoul/releases)

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fbjo163%2Frocksoul&project-name=rocksoul)

</div>

---

RockSoul is a **digital life / cognitive runtime**, not a single LLM. Its production runtime is Rust-first and is intended to grow through verified experience: identity, senses, memory, world state, reasoning, verification, tools, missions, XP, cognitive age and trust.

Human cognitive structure and selected Qur'anic principles are used as bounded architectural inspiration. RockSoul is **not** claimed to possess a human soul, `ruh`, `nafs`, consciousness, AGI or ASI.

## Live state

```text
┌──────────────────────────────────────────────────────────────┐
│                 ROCKSOUL • GEN 1 • AGE 0 • LV 1            │
├─ HOME ───────────────────────────────────────────────────────┤
│ Name             RockSoul                                   │
│ Cognitive Age    0                                          │
│ Level            1                                          │
│ XP               0                                          │
│ Trust            0                                          │
│ World            Internet                                   │
│ State            Born                                       │
│                                                              │
│ Phase A / Birth — brain not connected yet.                  │
├──────────────────────────────────────────────────────────────┤
│              Local-first digital cognitive runtime          │
└──────────────────────────────────────────────────────────────┘
```

The native **Ratatui TUI** and lightweight **Vercel web mirror** share display identity from [`public/brand.env`](public/brand.env). The browser surface intentionally adds no frontend framework in Phase A.

## Repository control plane

| Surface | Role | Source of truth |
| --- | --- | --- |
| `main` | releasable state | Git |
| `dev` | integration | Git |
| `feature/*` | human feature work | Git |
| Issues | executable units of work | `.github/roadmap-issues.json` for canonical roadmap issues |
| Labels | type / area / priority / phase | `.github/labels.json` |
| Milestones | Foundation + Phase A–M | `.github/milestones.json` |
| Public Project | Kanban + Roadmap + Backlog visualization | synchronized from Issues |
| Wiki | long-form browsable documentation | synchronized from `/docs` |
| Releases | age-aligned binaries and immutable bundles | Git tags + gate |
| Actions | quality, security, sync, release automation | `gate.yml` + `sync.yml` |

GitHub Projects supports board/Kanban, table and roadmap views; RockSoul uses all three without making the Project a second database.

## Two automation planes only

```text
rocksoul-gate
├── branch/repository policy
├── Rust fmt/check/clippy/test
├── RustSec
├── dependency review
├── Markdown + web smoke
├── CodeQL
└── tag → Linux / Windows EXE / macOS release binaries

rocksoul-sync
├── PR path labels
├── labels + milestones + canonical issues
├── public Project + fields + views
├── Wiki mirror
├── repository metadata/template settings
├── Release Drafter
├── age-aligned tag creation
└── stale hygiene
```

Dependabot stays GitHub-native and is the **only dependency bot**. No Renovate overlap. AI automation is not required for this control plane.

## Mental model

```text
WORLD
  ↓
SENSES
  ↓
ATTENTION / NANO
  ↓
MEMORY + KNOWLEDGE
  ↓
AQL / CORTEX
  ↓
FU'AD / JUDGMENT
  ↓
TABAYYUN / VERIFICATION
  ↓
ACTION
  ↓
WORLD
  ↓
MUHASABAH
  ↓
EXPERIENCE → XP → LEVEL
```

The Internet and authorized digital systems are modeled as an explicit world of places, entities, observations, provenance, trust and discovery state.

## Quick start

```bash
git clone https://github.com/bjo163/rocksoul.git
cd rocksoul
cargo run -p rocksoul-tui
```

Run the web mirror locally:

```bash
python -m http.server 3000 -d public
```

Then open `http://localhost:3000`.

## Downloadable releases

Age-aligned versions use:

```text
v0.<cognitive-age>.<revision>
```

For Age `0`, examples are `v0.0.1`, `v0.0.2`, and so on. Public tags trigger verified builds for:

- Linux x86_64
- Windows x86_64 `.exe`
- macOS arm64

See [Deployment](docs/DEPLOYMENT.md).

## Local-first storage

RockSoul does not treat every available cloud account as a mandatory dependency.

```text
LOCAL FIRST
├── local files / SQLite / local Postgres
│
├── GitHub
│   ├── code + config + small public manifests
│   ├── Issues / milestones / Project metadata
│   └── Releases → binaries / immutable bundles
│
├── Cloudflare R2 → preferred large public object storage
│
└── optional shared relational profile
    ├── Supabase → Postgres + Auth/RLS/Realtime/Storage when integrated features matter
    └── Neon     → Postgres when database-only cloud is preferable
```

**Supabase and Neon are alternatives for the same shared relational domain, not dual writable masters.** Vercel Blob is reserved for web-adjacent objects; R2 remains the general object-store preference.

See [Storage and Data Placement](docs/STORAGE.md).

## Repository boundary

| Repository | Responsibility |
| --- | --- |
| [`bjo163/rocksoul`](https://github.com/bjo163/rocksoul) | runtime, life, cognition, world, memory, tools, guardian, API, Ratatui, Web |
| [`bjo163/rocksoul-mind`](https://github.com/bjo163/rocksoul-mind) | model research, training, distillation, evaluation, checkpoint export |

Production consumes model artifacts behind a stable Rust interface and must not depend on training-repository Python internals.

## Structure

```text
rocksoul/
├── crates/
│   ├── rocksoul-core/
│   ├── rocksoul-life/
│   └── rocksoul-tui/
├── public/                 TUI/Web shared identity + zero-build web surface
├── data/public/            small public versioned data only
├── docs/                   canonical docs; Wiki mirrors these
├── .github/                machine-readable governance + two automation planes
├── vercel.json
├── Cargo.toml
└── rust-toolchain.toml
```

Planned crates are documented but are **not created empty in advance**.

## Documentation

Start at [`docs/README.md`](docs/README.md) or browse the [GitHub Wiki](https://github.com/bjo163/rocksoul/wiki).

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Development](docs/DEVELOPMENT.md)
- [Governance](docs/GOVERNANCE.md)
- [Automation](docs/AUTOMATION.md)
- [Project Management](docs/PROJECT_MANAGEMENT.md)
- [Storage](docs/STORAGE.md)
- [Deployment](docs/DEPLOYMENT.md)
- [Template / Fork Guide](docs/TEMPLATE.md)
- [Security](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## Engineering invariants

1. Sense before thinking.
2. Observation is not automatically fact.
3. Facts, beliefs and hypotheses remain distinct.
4. Important claims retain evidence and provenance.
5. Verify before trusting consequential information.
6. Intelligence never implies authority.
7. Tool execution is deterministic and policy-gated.
8. Only validated outcomes earn XP.
9. Cognitive age advances through evaluation, never time alone.
10. Live production experience never directly rewrites model weights.
11. One data domain has one authoritative store.
12. GitHub/Vercel/Wiki/Project are synchronized surfaces, never competing sources of truth.

## Template-ready

The runtime display identity lives in one file:

```text
public/brand.env
```

A template-derived project can change `NAME=RockSoul` to `NAME=StoneSoul` without introducing a parallel UI configuration. See [Template / Fork Guide](docs/TEMPLATE.md).

## Roadmap and ASI

Development proceeds through Birth, World, Memory, Nano, Skills, Cognitive Workspace, Guardian, Life progression, Cortex, Shura, autonomous research and controlled collective intelligence.

**ASI is a research gate, not a branding milestone.** Completing phases, raising age/level or increasing parameter count does not establish ASI.
