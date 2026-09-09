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
[![Local First](https://img.shields.io/badge/architecture-local--first-0e8a16)](docs/STORAGE.md)
[![Template Ready](https://img.shields.io/badge/world-template--ready-5319e7)](docs/TEMPLATE.md)

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

RockSoul is a **digital life / cognitive runtime**, not a single LLM and not a visual-only metaverse. The Rust runtime is intended to grow through verifiable state: identity, senses, world, memory, knowledge, reasoning, verification, tools, missions, XP, cognitive age and trust.

The RPG/world vocabulary is an interface over real engineering objects. **Issue = Quest** does not create a second task database. **Runner = World Node** does not make an unavailable runner appear online. **Trust = verification policy**, not decoration.

Human cognitive structure and selected Qur'anic principles are used as bounded architectural inspiration. RockSoul is **not** claimed to possess a human soul, `ruh`, `nafs`, consciousness, AGI or ASI.

## ENTER WORLD

The shared Operator / World interface now uses the same core navigation in Ratatui and Web:

```text
┌─ ROCKSOUL OPERATOR ─────────────────────────────────────────┐
│ ROCKSOUL • GEN 1 • AGE 0 • LV 1                            │
├─────────────────────────────────────────────────────────────┤
│ [1 WORLD] [2 MAP] [3 QUESTS] [4 CODEX] [5 SYSTEM]          │
├─────────────────────────────────────────────────────────────┤
│ WORLD: INTERNET                                              │
│ STATE: BORN                                                  │
│                                                              │
│ WORLD / MAP / QUESTS / KNOWLEDGE / SYSTEMS                  │
│ all follow the same evidence and topology contract.          │
├─────────────────────────────────────────────────────────────┤
│ evidence before trust • q / Esc exit                         │
└─────────────────────────────────────────────────────────────┘
```

Shared state contracts:

- [`public/brand.env`](public/brand.env) — identity, Age, Level, XP, Trust, navigation and accepted visual revisions.
- [`public/world.json`](public/world.json) — topology, evidence vocabulary, discovery progression and World terminology.
- [`docs/WORLD.md`](docs/WORLD.md) — human-readable World/Codex contract.

The Web World Surface can observe public GitHub data for repositories, Issues, Releases and repository events. The native TUI deliberately does **not** pretend to have those live network observations before its Rust Sense/API connector exists.

## WORLD STATUS

Status words are intentionally strict:

```text
VERIFIED  OBSERVED  INFERRED  PENDING  BLOCKED  UNKNOWN
```

Current public topology is designed to report only what can actually be evidenced:

| World object | Engineering object | Current evidence contract |
| --- | --- | --- |
| World Core | `bjo163/rocksoul` | OBSERVED |
| World Resource | `bjo163/rocksoul-assets` | OBSERVED |
| UI Grammar | `bjo163/rocksoul-ui` | OBSERVED |
| Cognition Lab | `bjo163/rocksoul-mind` | OBSERVED |
| World Map | public GitHub Project `rocksoul` | PENDING privileged sync |
| Archive | GitHub Wiki mirror | PENDING privileged sync |
| Codex Portal | GitHub Pages | PENDING until Pages API + deployment prove activation |
| Live World Portal | canonical Vercel project for `bjo163/rocksoul` | PENDING |
| World Nodes | self-hosted runner inventory for this repository | UNKNOWN until authenticated evidence exists |
| Gateway | Cloudflare Tunnel | PENDING / not required without a private service exposure need |

A configuration file is not proof that a deployment is online. These states change only after evidence changes.

## WORLD MAP

```text
                           ROCKSOUL WORLD
                                │
               ┌────────────────┼────────────────┐
               │                │                │
           WORLD CORE       WORLD SYSTEMS     WORLD NODES
             rocksoul        gate / sync       runners
               │             cognition         services
               │             memory
               │             knowledge
               │             trust
               │             experience
       ┌───────┼──────────┬─────────────┐
       │       │          │             │
    ASSETS     UI        MIND         PORTALS
       │       │          │             │
 resources  grammar    cognition   Pages / Vercel
                                    Cloudflare only
                                    when justified
```

The current map is topology-first. A geographic or richer metaverse map arrives only when actual World Graph place/route data exists. We do not add a 3D engine to simulate maturity.

### Fog of War / discovery

```text
UNKNOWN
  ↓
DISCOVERED
  ↓
OBSERVED
  ↓
VISITED
  ↓
STUDIED
  ↓
VERIFIED
  ↓
TRUSTED
```

`TRUSTED` requires an explicit provenance/verification policy. Repeated visits do not automatically create trust.

## QUESTS & CHAPTERS

GitHub remains the engineering and world-building hub:

```text
GitHub Issue       → QUEST
Milestone          → CHAPTER / REGION
GitHub Project     → WORLD MAP
Pull Request       → CONTRIBUTION
Release            → WORLD BUILD
Actions            → WORLD SYSTEMS
```

Issues are the source of truth. The Project, Web Portal and future TUI Quest feed are views over those Issues—not independent task databases.

Canonical machine-readable governance:

- [`.github/labels.json`](.github/labels.json)
- [`.github/milestones.json`](.github/milestones.json)
- [`.github/roadmap-issues.json`](.github/roadmap-issues.json)

Labels encode `type:*`, `area:*`, `priority:*`, `status:*`, `phase:*` and release intent. Milestones represent Foundation and Phase A–M.

## CODEX & ARCHIVE

`/docs` is canonical knowledge. Public surfaces render it differently:

```text
docs/                 → canonical knowledge
   ├─ WORLD            → Digital World semantics
   ├─ ARCHITECTURE     → cognitive/runtime boundaries
   ├─ ROADMAP          → chapters / progression
   ├─ PROJECT          → quests / map
   ├─ STORAGE          → data placement
   ├─ AUTOMATION       → World Systems
   └─ DEPLOYMENT       → portals / World Builds
          │
          ├──────────→ GitHub Wiki  = ARCHIVE
          └──────────→ GitHub Pages = CODEX
```

Pages and Vercel intentionally reuse the same lightweight `public/` artifact. On `github.io`, the app enters **CODEX**; on Vercel/ordinary hosts it enters **WORLD**. That keeps one UI/state contract instead of two websites drifting apart.

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

The Internet and authorized digital systems are modeled as an explicit World of entities, places, relations, observations, provenance, trust and discovery state.

Architecture details: [Architecture](docs/ARCHITECTURE.md) and [World](docs/WORLD.md).

## EXPERIENCE / AGE / LEVEL

Progression is not click-based gamification:

```text
ACTION
  ↓
OUTCOME
  ↓
VALIDATION
  ↓
VERIFIED EXPERIENCE
  ↓
XP
  ↓
LEVEL / CAPABILITY
```

Only validated outcomes earn success XP. Cognitive Age advances through evaluation gates, not elapsed wall-clock time. Trust never bypasses authorization policy.

## WORLD RESOURCES — ASSETS × UI

RockSoul does not create a third design system.

```text
bjo163/rocksoul-assets
          ↓
     @rocksoul/ui
          ↓
 RockSoul World surfaces
```

- `rocksoul-assets` is the visual source of truth: architecture nodes, geospatial/map primitives, status/badge assets, Kanban, graph, command/keyboard, cinematic and runtime packs.
- `rocksoul-ui` converts that visual truth into production UI contracts and reusable components.
- this runtime consumes the accepted visual contract instead of forking it.

The currently accepted asset/UI revisions are pinned in `public/brand.env` and `public/world.json`. A newer asset `main` head is not automatically promoted just because it exists.

## WORLD SYSTEMS — TWO AUTOMATION PLANES

```text
rocksoul-gate                         rocksoul-sync
├── branch policy                     ├── PR labels
├── world/brand contract              ├── labels / milestones / Issues
├── Rust fmt/check/clippy/test         ├── public Project / World Map
├── TUI contract tests                ├── Wiki / Archive
├── RustSec                           ├── Pages / Codex
├── dependency review                 ├── repo metadata/template
├── Portal HTTP smoke                 ├── Release Drafter
├── CodeQL                            ├── age-aligned release request
└── World Build binaries              └── stale hygiene
```

Dependabot is the only dependency-update bot. No Renovate overlap. No permanent AI reviewer is installed because the current synchronization/classification work is deterministic.

If a real semantic automation gap appears later, one AI gateway may be added behind `rocksoul-sync`; it should not become a competing third control plane.

See [Automation](docs/AUTOMATION.md).

## PORTALS

### GitHub Pages — CODEX

Expected URL after verified activation:

`https://bjo163.github.io/rocksoul/`

Pages remains **PENDING** until GitHub reports the site configured and the deployment succeeds.

### Vercel — LIVE WORLD PORTAL

The repository has zero-build [`vercel.json`](vercel.json) configuration for `public/`.

Canonical Vercel deployment must be linked to **this repository**, `bjo163/rocksoul`. An existing `rocksoul-*` Vercel project connected to another repository is not treated as the RockSoul World Portal.

### Cloudflare — GATEWAY

No Tunnel is required merely because RockSoul has a World concept. GitHub Pages/Vercel already serve static public surfaces.

A tunnel is introduced only for a justified private service:

```text
PRIVATE NODE → ROCKSOUL SERVICE → CLOUDFLARE TUNNEL → AUTHORIZED GATEWAY
```

Default: **PRIVATE FIRST**.

## LOCAL-FIRST STORAGE

RockSoul does not turn every available cloud account into a dependency.

```text
LOCAL FIRST
├── local files / SQLite / local Postgres
│
├── GitHub
│   ├── code + configuration + small public manifests
│   ├── Issues / Milestones / Project metadata
│   └── Releases → binaries / immutable bundles
│
├── Cloudflare R2 → preferred general large-object target when provisioned
│
└── optional shared relational profile
    ├── Supabase → Postgres + integrated Auth/RLS/Realtime/Storage
    └── Neon     → focused serverless Postgres profile
```

**Supabase and Neon are alternatives for a shared relational domain, not dual writable masters.** Vercel Blob is reserved for web-adjacent objects. Architecture documentation is not evidence that any remote bucket/database has already been provisioned.

See [Storage and Data Placement](docs/STORAGE.md).

## WORLD BUILDS / DOWNLOADABLE EXE

Distribution versions are aligned to evaluated Cognitive Age:

```text
v0.<cognitive-age>.<revision>
```

A release is opt-in: merge a PR labeled `release:ready`, or manually request `publish_age_release` in `rocksoul-sync`.

The exact release tag is revalidated by `rocksoul-gate`, then produces:

- Linux x86_64 archive
- Windows x86_64 archive containing `rocksoul.exe`
- macOS arm64 archive

A build is not described as released until the GitHub Release and assets actually exist.

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

Those commands are usage instructions. They are not a claim that the repository-platform work used or prepared a local clone.

## REPOSITORY BOUNDARIES

| Repository | World role | Responsibility |
| --- | --- | --- |
| [`bjo163/rocksoul`](https://github.com/bjo163/rocksoul) | WORLD CORE | runtime, life, World, cognition orchestration, tools, guardian, TUI/Web |
| [`bjo163/rocksoul-assets`](https://github.com/bjo163/rocksoul-assets) | WORLD RESOURCE | canonical visual assets/design source |
| [`bjo163/rocksoul-ui`](https://github.com/bjo163/rocksoul-ui) | WORLD SYSTEM / UI GRAMMAR | production visual implementation contract |
| [`bjo163/rocksoul-mind`](https://github.com/bjo163/rocksoul-mind) | COGNITION LAB | model research, training, distillation, evaluation, export |

Production model integration must remain behind a stable Rust interface; the runtime should not depend on training-repository Python internals.

## REPOSITORY STRUCTURE

```text
rocksoul/
├── crates/
│   ├── rocksoul-core/
│   ├── rocksoul-life/
│   └── rocksoul-tui/
├── public/
│   ├── brand.env           shared identity / accepted revisions
│   ├── world.json          shared World topology / evidence contract
│   ├── index.html          World / Codex surface
│   ├── styles.css
│   └── app.js
├── data/public/            small public versioned data only
├── docs/                   canonical Codex; Wiki mirrors these
├── .github/                machine governance + gate/sync planes
├── vercel.json
├── Cargo.toml
└── rust-toolchain.toml
```

Planned crates are documented but are **not created empty in advance**.

## CODEX INDEX

Start at [`docs/README.md`](docs/README.md).

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

Runtime identity is centralized in:

```text
public/brand.env
```

A template-derived project can begin as `StoneSoul` or another world by changing the identity contract instead of maintaining separate TUI/Web branding. See [Template / Fork Guide](docs/TEMPLATE.md).

## ROADMAP & ASI

Development proceeds through Birth, World, Memory, Nano, Skills, Cognitive Workspace, Guardian, Life progression, Cortex, Shura, autonomous research and controlled collective intelligence.

**ASI is a research gate, not a branding milestone.** Completing the World, raising Age/Level, adding agents, or increasing parameter count does not establish ASI.
