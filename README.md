<div align="center">

<img src="https://raw.githubusercontent.com/bjo163/rocksoul-assets/5b035d0d83612f847ccd8b35f2c1b04022500110/moonwitness/brand/rocksoul-lockup.svg" alt="RockSoul ecosystem" width="720">

# ROCKSOUL WORLD

**Local-first Rust digital cognitive runtime — a navigable digital world built from real systems, evidence, quests, knowledge and verified experience.**

[![Gate](https://img.shields.io/github/actions/workflow/status/bjo163/rocksoul/gate.yml?branch=main&label=trust%20gate&logo=githubactions)](https://github.com/bjo163/rocksoul/actions/workflows/gate.yml)
[![Release](https://img.shields.io/github/v/release/bjo163/rocksoul?display_name=tag&sort=semver&label=world%20build)](https://github.com/bjo163/rocksoul/releases)
[![Downloads](https://img.shields.io/github/downloads/bjo163/rocksoul/total?label=downloads)](https://github.com/bjo163/rocksoul/releases)
[![Issues](https://img.shields.io/github/issues/bjo163/rocksoul?label=quests)](https://github.com/bjo163/rocksoul/issues)
[![Stars](https://img.shields.io/github/stars/bjo163/rocksoul?style=flat&label=stars)](https://github.com/bjo163/rocksoul/stargazers)
[![Forks](https://img.shields.io/github/forks/bjo163/rocksoul?style=flat&label=forks)](https://github.com/bjo163/rocksoul/forks)
[![Last Commit](https://img.shields.io/github/last-commit/bjo163/rocksoul?label=last%20observation)](https://github.com/bjo163/rocksoul/commits/main)
[![Rust](https://img.shields.io/badge/Rust-1.88-000000?logo=rust)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Local First](https://img.shields.io/badge/architecture-local--first-0e8a16)](docs/STORAGE.md)
[![Template Ready](https://img.shields.io/badge/world-template--ready-5319e7)](docs/TEMPLATE.md)

[**LIVE WORLD PORTAL**](https://rocksoul-bjo163s-projects.vercel.app) ·
[**ENTER CODEX**](https://bjo163.github.io/rocksoul/) ·
[**QUESTS**](https://github.com/bjo163/rocksoul/issues) ·
[**WORLD MAP**](https://github.com/users/bjo163/projects?query=rocksoul) ·
[**ARCHIVE**](https://github.com/bjo163/rocksoul/wiki) ·
[**DISCUSSIONS**](https://github.com/bjo163/rocksoul/discussions) ·
[**WORLD SYSTEMS**](https://github.com/bjo163/rocksoul/actions) ·
[**WORLD BUILDS / EXE**](https://github.com/bjo163/rocksoul/releases)

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fbjo163%2Frocksoul&project-name=rocksoul)

</div>

---

RockSoul is a **digital life / cognitive runtime**, not a single LLM and not a visual-only metaverse. The runtime is Rust-first and is intended to grow through verifiable state: identity, senses, world, memory, knowledge, reasoning, verification, tools, missions, XP, cognitive age and trust.

The RPG vocabulary is a presentation layer over real engineering objects:

```text
GitHub Issue       → QUEST
Milestone          → CHAPTER / REGION
GitHub Project     → WORLD MAP
Pull Request       → CONTRIBUTION
Release            → WORLD BUILD
Actions / runners  → WORLD SYSTEMS / WORLD NODES
Documentation      → CODEX
Verification       → TRUST GATE
```

Human cognitive structure and selected Qur'anic principles are used as bounded architectural inspiration. RockSoul is **not** claimed to possess a human soul, `ruh`, `nafs`, consciousness, AGI or ASI.

## ENTER WORLD

Ratatui and Web share one navigation/state contract:

```text
┌─ ROCKSOUL OPERATOR ─────────────────────────────────────────┐
│ ROCKSOUL • GEN 1 • AGE 0 • LV 1                            │
├─────────────────────────────────────────────────────────────┤
│ [1 WORLD] [2 MAP] [3 QUESTS] [4 CODEX] [5 SYSTEM]          │
├─────────────────────────────────────────────────────────────┤
│ WORLD: INTERNET                                              │
│ STATE: BORN                                                  │
│                                                              │
│ evidence before trust • local first • finite authority       │
└─────────────────────────────────────────────────────────────┘
```

Shared contracts:

- [`public/brand.env`](public/brand.env) — identity, Age, Level, XP, Trust, repository/portal identity and accepted revisions.
- [`public/world.json`](public/world.json) — topology, evidence vocabulary, discovery progression and World terminology.
- [`docs/WORLD.md`](docs/WORLD.md) — human-readable Digital World contract.

The browser World Surface observes public GitHub repositories, Issues, Releases and repository events. The native TUI deliberately does **not** invent network observations before its Rust Sense/API connector exists.

## WORLD STATUS

Evidence vocabulary is deliberately strict:

```text
VERIFIED  OBSERVED  INFERRED  PENDING  BLOCKED  UNKNOWN
```

| World object | Engineering object | Evidence |
| --- | --- | --- |
| World Core | `bjo163/rocksoul` | OBSERVED |
| World Resource | `bjo163/rocksoul-assets` | OBSERVED |
| UI Grammar | `bjo163/rocksoul-ui` | OBSERVED |
| Cognition Lab | `bjo163/rocksoul-mind` | OBSERVED |
| Live World Portal | Vercel project `rocksoul` linked to this repo | **VERIFIED** |
| World Map | public GitHub Project `rocksoul` | PENDING privileged sync |
| Archive | GitHub Wiki mirror | PENDING privileged sync |
| Codex Portal | GitHub Pages | PENDING verified Pages deployment |
| World Nodes | self-hosted runner inventory | UNKNOWN until authenticated runner evidence is captured |
| Gateway | Cloudflare Tunnel | PENDING / unnecessary until a private service needs exposure |

Vercel evidence was verified on **2026-09-09**: the connected Vercel account reports project `rocksoul` linked to `bjo163/rocksoul` with a `READY` production deployment from `main`.

A configuration file alone never upgrades an object to VERIFIED/ONLINE/TRUSTED.

## WORLD MAP & FOG OF WAR

```text
                           ROCKSOUL WORLD
                                │
               ┌────────────────┼────────────────┐
               │                │                │
           WORLD CORE       WORLD SYSTEMS     WORLD NODES
               │                │                │
       ┌───────┼────────┐    gate / sync       runners
       │       │        │
    ASSETS     UI      MIND
       │       │        │
       └───────┴────────┴────────→ PORTALS
                                Pages / Vercel
```

Discovery progression:

```text
UNKNOWN → DISCOVERED → OBSERVED → VISITED → STUDIED → VERIFIED → TRUSTED
```

`TRUSTED` requires an explicit provenance/verification policy. Repeated visits alone do not create trust.

## COGNITIVE SYSTEM

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
VERIFIED EXPERIENCE → XP → LEVEL
```

Only validated outcomes earn success XP. Cognitive Age advances through evaluation gates, not elapsed time. Trust never bypasses authorization policy.

See [Architecture](docs/ARCHITECTURE.md), [World](docs/WORLD.md) and [Roadmap](docs/ROADMAP.md).

## CODEX / PROJECT / QUESTS

Canonical sources remain simple:

```text
/docs                       canonical long-form knowledge
GitHub Issues               canonical executable work
GitHub Milestones           canonical phase/chapter grouping
.github/labels.json          label taxonomy
.github/roadmap-issues.json  canonical seeded Issues
public/brand.env             shared identity
public/world.json            shared topology/evidence contract
```

Synchronized views:

```text
README           → WORLD ENTRY
GitHub Pages     → CODEX
GitHub Wiki      → ARCHIVE
GitHub Project   → WORLD MAP / KANBAN / ROADMAP / BACKLOG
Vercel           → LIVE WORLD PORTAL
Ratatui          → OPERATOR WINDOW
GitHub Releases  → WORLD BUILDS
```

The views never become competing databases.

## WORLD RESOURCES — ASSETS × UI

RockSoul deliberately avoids creating a third design system:

```text
bjo163/rocksoul-assets
          ↓
     @rocksoul/ui
          ↓
 RockSoul World surfaces
```

- `rocksoul-assets` is the canonical visual resource source.
- `rocksoul-ui` is the canonical production UI grammar.
- RockSoul consumes pinned accepted revisions from `brand.env` / `world.json`.
- a newer asset head is not promoted merely because it exists.

## WORLD SYSTEMS — TWO AUTOMATION PLANES

```text
rocksoul-gate                         rocksoul-sync
├── branch policy                     ├── PR labels
├── world/brand/license contract      ├── labels / milestones / Issues
├── Rust fmt/check/clippy/test         ├── public Project / World Map
├── TUI contract tests                ├── Wiki / Archive
├── RustSec                           ├── Pages / Codex
├── dependency review                 ├── repo metadata/template
├── Portal HTTP smoke                 ├── Release Drafter
├── CodeQL                            ├── age-aligned release request
└── World Build binaries              └── stale hygiene
```

Dependabot is the **only dependency-update bot**. No Renovate overlap. No AI reviewer is installed because current governance/sync work is deterministic.

If a measurable semantic automation gap appears later, one AI gateway may be added behind `rocksoul-sync`; it should not become a competing third control plane.

See [Automation](docs/AUTOMATION.md).

## LOCAL-FIRST STORAGE

```text
LOCAL FIRST
├── local files / SQLite / local Postgres
│
├── GitHub
│   ├── code + configuration + small public manifests
│   ├── Issues / Milestones / Project metadata
│   └── Releases → binaries / immutable bundles
│
├── Cloudflare R2 → preferred large-object target when provisioned
│
└── optional shared relational profile
    ├── Supabase → integrated Postgres/Auth/RLS/Realtime/Storage
    └── Neon     → focused serverless Postgres profile
```

Supabase and Neon are **alternatives for one shared relational domain**, not dual writable masters. Vercel/Pages are presentation surfaces, not databases.

See [Storage and Data Placement](docs/STORAGE.md).

## WORLD BUILDS / DOWNLOADABLE EXE

Distribution versions follow evaluated Cognitive Age:

```text
v0.<cognitive-age>.<revision>
```

A release is opt-in: merge a PR labeled `release:ready`, or manually request `publish_age_release` in `rocksoul-sync`.

The exact tag is revalidated by `rocksoul-gate`, then produces:

- Linux x86_64 archive
- Windows x86_64 archive containing `rocksoul.exe`
- macOS arm64 archive

A build is not called released until the GitHub Release and assets actually exist.

See [Deployment](docs/DEPLOYMENT.md).

## QUICK START

```bash
git clone https://github.com/bjo163/rocksoul.git
cd rocksoul
cargo run -p rocksoul-tui
```

Optional local World Surface:

```bash
python -m http.server 3000 -d public
```

## REPOSITORY BOUNDARIES

| Repository | World role | Responsibility |
| --- | --- | --- |
| [`bjo163/rocksoul`](https://github.com/bjo163/rocksoul) | WORLD CORE | runtime, life, World, cognition orchestration, tools, guardian, TUI/Web |
| [`bjo163/rocksoul-assets`](https://github.com/bjo163/rocksoul-assets) | WORLD RESOURCE | canonical visual assets/design source |
| [`bjo163/rocksoul-ui`](https://github.com/bjo163/rocksoul-ui) | UI GRAMMAR | production visual implementation contract |
| [`bjo163/rocksoul-mind`](https://github.com/bjo163/rocksoul-mind) | COGNITION LAB | model research, training, distillation, evaluation, export |

Production model integration remains behind a stable Rust interface; runtime must not depend on training-repository Python internals.

## REPOSITORY STRUCTURE

```text
rocksoul/
├── crates/
│   ├── rocksoul-core/
│   ├── rocksoul-life/
│   └── rocksoul-tui/
├── public/
│   ├── brand.env
│   ├── world.json
│   ├── index.html
│   ├── styles.css
│   └── app.js
├── data/public/
├── docs/
├── .github/
├── LICENSE                Apache-2.0
├── vercel.json
├── Cargo.toml
└── rust-toolchain.toml
```

Planned crates are documented but are **not created empty in advance**.

## CODEX INDEX

- [World](docs/WORLD.md)
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
- [Apache-2.0 License](LICENSE)

## ENGINEERING INVARIANTS

1. Sense before thinking.
2. Observation is not automatically fact.
3. Facts, beliefs and hypotheses remain distinct.
4. Important claims retain evidence and provenance.
5. Verify before trusting consequential information.
6. Intelligence never implies authority.
7. Tool execution is deterministic and policy-gated.
8. Only validated outcomes earn XP.
9. Cognitive Age advances through evaluation, never time alone.
10. Live production experience never directly rewrites model weights.
11. One data domain has one authoritative store.
12. GitHub/Pages/Vercel/Wiki/Project are synchronized views, never competing sources of truth.
13. A visual state never upgrades beyond its evidence state.
14. A World Node is never shown ONLINE/READY without actual runner evidence.

## TEMPLATE-READY WORLDS

Runtime/repository identity is centralized in [`public/brand.env`](public/brand.env), while topology uses brand-neutral tokens in [`public/world.json`](public/world.json).

A template-derived project can begin as **StoneSoul** or another world without maintaining separate TUI/Web branding or rewriting the automation architecture. See [Template / Fork Guide](docs/TEMPLATE.md).

## LICENSE

RockSoul is licensed under the **Apache License 2.0**. See [`LICENSE`](LICENSE).

The license is permissive and includes an explicit patent grant. Third-party dependencies, model artifacts and referenced ecosystem assets retain their own applicable licenses and provenance.

## ROADMAP & ASI

Development proceeds through Birth, World, Memory, Nano, Skills, Cognitive Workspace, Guardian, Life progression, Cortex, Shura, autonomous research and controlled collective intelligence.

**ASI is a research gate, not a branding milestone.** Completing the World, raising Age/Level, adding agents or increasing parameter count does not establish ASI.
