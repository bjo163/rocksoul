# RockSoul Documentation

This directory is the **canonical long-form documentation** for RockSoul. The GitHub Wiki mirrors these files for browsing; edits belong here first so Wiki, README, automation and repository state do not drift.

GitHub Issues and milestones track executable work. The public GitHub Project is a synchronized Kanban/Roadmap/Backlog view over those Issues, not another source of truth.

## Start here

| Document | Purpose |
| --- | --- |
| [WORLD.md](WORLD.md) | Digital World semantics, World Map, quests, discovery, evidence states, portals and asset provenance |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System boundary, cognitive model, world model, life model, invariants |
| [ROADMAP.md](ROADMAP.md) | Phase order and exit criteria |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Local development and crate rules |
| [GOVERNANCE.md](GOVERNANCE.md) | Branches, labels, Issues, milestones, PR and release policy |
| [AUTOMATION.md](AUTOMATION.md) | `rocksoul-gate`, `rocksoul-sync`, Dependabot and overlap prevention |
| [PROJECT_MANAGEMENT.md](PROJECT_MANAGEMENT.md) | Public Project views, labels, milestones and synchronization model |
| [STORAGE.md](STORAGE.md) | Local-first data placement, GitHub, R2, Supabase, Neon and Vercel Blob |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Ratatui/Web parity, Pages Codex, Vercel Portal and age-aligned executable releases |
| [TEMPLATE.md](TEMPLATE.md) | Reuse the repository as RockSoul, StoneSoul or another derived runtime |

Repository-level policies:

- [../CONTRIBUTING.md](../CONTRIBUTING.md)
- [../SECURITY.md](../SECURITY.md)
- [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)

## Source-of-truth map

| Concern | Canonical source | Synchronized surface |
| --- | --- | --- |
| code | `main` | GitHub Releases / public deployments |
| Digital World topology | `public/world.json` + verified runtime evidence | Ratatui + Web World Map |
| architecture and invariants | `docs/ARCHITECTURE.md` | Wiki / Pages Codex |
| World semantics | `docs/WORLD.md` | Wiki / Pages Codex |
| phase intent/order | `docs/ROADMAP.md` | Wiki / Project Roadmap |
| executable work | GitHub Issues | Project Kanban/Backlog / Web Quests |
| phase grouping | GitHub Milestones | Project Roadmap |
| label taxonomy | `.github/labels.json` | GitHub labels / Project fields |
| canonical roadmap Issues | `.github/roadmap-issues.json` | GitHub Issues |
| runtime display identity | `public/brand.env` | Ratatui + Web |
| accepted visual resources | pinned `rocksoul-assets` / `rocksoul-ui` revisions | Web/TUI presentation contracts |
| public small data | `data/public/` | Git history |
| large immutable artifacts | local source + manifest | GitHub Releases / Cloudflare R2 when configured |
| model research | `bjo163/rocksoul-mind` | exported artifacts only |

## Public surfaces

```text
README           → WORLD ENTRY
GitHub Pages     → CODEX
GitHub Wiki      → ARCHIVE
GitHub Project   → WORLD MAP
GitHub Issues    → QUESTS
GitHub Releases  → WORLD BUILDS
Vercel           → LIVE WORLD PORTAL
Ratatui          → OPERATOR WINDOW
```

Those surfaces are not independent databases. They render or synchronize canonical repository/runtime state.

## Editing rule

Do not hand-maintain duplicate Wiki pages, Project metadata, Pages content or generated release notes. Change the canonical repository source and let `rocksoul-sync` update the GitHub surfaces.

When docs and implementation disagree, open an Issue and fix the inconsistency explicitly rather than silently treating one as correct.
