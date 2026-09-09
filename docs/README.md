# RockSoul Documentation

This directory is the **canonical long-form documentation** for RockSoul. The GitHub Wiki mirrors these files for browsing; edits belong here first so Wiki, README, automation and repository state do not drift.

GitHub issues and milestones track executable work. The public GitHub Project is a synchronized Kanban/Roadmap/Backlog view over those issues, not another source of truth.

## Start here

| Document | Purpose |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System boundary, cognitive model, world model, life model, invariants |
| [ROADMAP.md](ROADMAP.md) | Phase order and exit criteria |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Local development and crate rules |
| [GOVERNANCE.md](GOVERNANCE.md) | Branches, labels, issues, milestones, PR and release policy |
| [AUTOMATION.md](AUTOMATION.md) | `rocksoul-gate`, `rocksoul-sync`, Dependabot and overlap prevention |
| [PROJECT_MANAGEMENT.md](PROJECT_MANAGEMENT.md) | Public Project views, labels, milestones and synchronization model |
| [STORAGE.md](STORAGE.md) | Local-first data placement, GitHub, R2, Supabase, Neon and Vercel Blob |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Ratatui/Web parity, Vercel and age-aligned executable releases |
| [TEMPLATE.md](TEMPLATE.md) | Reuse the repository as RockSoul, StoneSoul or another derived runtime |

Repository-level policies:

- [../CONTRIBUTING.md](../CONTRIBUTING.md)
- [../SECURITY.md](../SECURITY.md)
- [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)

## Source-of-truth map

| Concern | Canonical source | Synchronized surface |
| --- | --- | --- |
| code | `main` | GitHub Releases / Vercel deployment |
| architecture and invariants | `docs/ARCHITECTURE.md` | Wiki |
| phase intent/order | `docs/ROADMAP.md` | Wiki / Project roadmap |
| executable work | GitHub Issues | Project Kanban/Backlog |
| phase grouping | GitHub Milestones | Project roadmap |
| label taxonomy | `.github/labels.json` | GitHub labels / Project fields |
| canonical roadmap issues | `.github/roadmap-issues.json` | GitHub Issues |
| runtime display identity | `public/brand.env` | Ratatui + Web |
| public small data | `data/public/` | Git history |
| large immutable artifacts | local source + manifest | GitHub Releases / Cloudflare R2 |
| model research | `bjo163/rocksoul-mind` | exported artifacts only |

## Editing rule

Do not hand-maintain duplicate Wiki pages, Project metadata or generated release notes. Change the canonical repository source and let `rocksoul-sync` update the GitHub surfaces.

When docs and implementation disagree, open an issue and fix the inconsistency explicitly rather than silently treating one as correct.
