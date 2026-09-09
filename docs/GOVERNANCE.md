# Repository Governance

RockSoul uses machine-readable repository governance so GitHub surfaces can be rebuilt instead of maintained by hand.

## Sources of truth

| Concern | Source of truth |
| --- | --- |
| releasable code | `main` |
| integration code | `dev` |
| feature work | `feature/*` |
| architecture | `docs/ARCHITECTURE.md` |
| Digital World semantics | `docs/WORLD.md` |
| World topology/evidence vocabulary | `public/world.json` |
| phase intent | `docs/ROADMAP.md` |
| executable work / Quests | GitHub Issues |
| phase grouping / Chapters | GitHub Milestones |
| label taxonomy | `.github/labels.json` |
| canonical roadmap/Foundation Issues | `.github/roadmap-issues.json` |
| automation | `gate.yml`, `sync.yml`, Dependabot |
| TUI/Web identity and repository source | `public/brand.env` |

## Branch policy

The intended branch namespace is deliberately small:

```text
main
  stable releasable source

dev
  integration branch

feature/*
  short-lived human work

dependabot/*
  short-lived GitHub-native dependency bot exception
```

Do not create new `release/*`, `hotfix/*`, `chore/*`, `docs/*`, or long-lived experiment branches. Use `feature/<purpose>` and labels to describe work type.

`rocksoul-gate` rejects pull requests from human branch names outside `dev` and `feature/*`.

Legacy `chore/*` refs created before this policy are tracked for explicit cleanup. Do not call the branch policy complete until those refs are removed or proven necessary.

## `main` protection

`main` should be protected with a GitHub ruleset requiring:

- pull request before merge;
- required `rocksoul-gate` checks;
- no force push;
- no protected-branch deletion;
- squash merge as the normal history strategy.

Branch protection is an actual GitHub admin setting. Automation must not claim it exists until GitHub reports the branch/ruleset protected.

## Merge strategy

- **Squash merge:** enabled and preferred.
- **Merge commit:** disabled once privileged repository sync is active.
- **Rebase merge:** disabled once privileged repository sync is active.
- **Delete branch on merge:** enabled once privileged repository sync is active.

This keeps public history readable and makes feature-branch cleanup predictable.

## Labels / tags

Labels are namespaced:

- `type:*` — bug, feature, docs, architecture, maintenance, security, research;
- `area:*` — core, life, TUI, Web, World, Memory, Brain, Senses, Skills, Cognition, Guardian, Storage, CI;
- `priority:p0..p3` — urgency;
- `status:*` — ready, needs-design, blocked;
- `phase:*` — Phase A–M;
- `release:*` — release intent;
- meta labels such as `dependencies`, `breaking-change`, `stale`, `roadmap`.

Repository discovery topics are separate from Issue labels. Privileged `rocksoul-sync` owns topics so README/About metadata do not drift.

## Issues and milestones

GitHub Issues are the canonical public work database. The RPG term **Quest** is a presentation alias only.

Each canonical work item should have:

- one clear goal;
- scope and non-goals;
- measurable acceptance criteria;
- dependencies/blockers when known;
- one Foundation/phase milestone;
- labels for type, area, priority and phase/status.

Milestones are the canonical phase grouping; **Chapter / Region** is their World terminology.

The public Project is a visualization over these Issues. Moving a Project card must not invent a second project model that contradicts Issue labels or milestones.

## Pull requests

PRs should be coherent rather than artificially tiny. One Contribution may update implementation, docs, tests and automation when those changes are one logical unit.

Path labeling is automatic. `rocksoul-gate` is the deterministic merge-quality/Trust Gate.

## Public Project, Wiki and Pages

These are synchronized surfaces:

```text
GitHub Project → WORLD MAP
GitHub Wiki    → ARCHIVE
GitHub Pages   → CODEX
```

- Issues + labels + milestones remain canonical for project work.
- `/docs` remains canonical for long-form knowledge.
- `public/` remains canonical for the static World/Codex surface.

A single optional `ROCKSOUL_GITHUB_TOKEN` activates privileged Project/Wiki/repository/Pages settings. If it is absent, the sync jobs report the gap; they do not fake a completed surface.

## Age-aligned World Builds

RockSoul does not release every merge.

A public World Build is requested in one of two ways:

1. merge a PR carrying `release:ready`; or
2. manually run `rocksoul-sync` with `publish_age_release=true`.

The sync plane creates the next:

```text
v0.<cognitive-age>.<revision>
```

It then explicitly dispatches `rocksoul-gate` on that tag. The gate repeats validation before creating the GitHub Release and Linux/Windows/macOS binaries.

`release:skip` documents that a change should not become a public World Build.

Cognitive Age is an evaluated runtime/project concept. The version mirrors that state; the tag is not evidence of intelligence or maturity.

No release branches are required.

## World evidence policy

Public UI states use:

```text
VERIFIED  OBSERVED  INFERRED  PENDING  BLOCKED  UNKNOWN
```

Do not present `ONLINE`, `READY`, `TRUSTED`, or similar stronger states unless the relevant source proves them.

This especially applies to:

- self-hosted runners / World Nodes;
- Vercel deployments / Portals;
- Cloudflare gateways;
- Wiki/Pages/Project activation;
- runtime events and trust claims.

## Template policy

The repository is intended to be reusable as a GitHub template. Product/repository identity belongs in `public/brand.env`; World topology uses brand-neutral `public/world.json` tokens. Automation should not require mass-copy edits for a derivative such as StoneSoul.

See `docs/TEMPLATE.md`.

## Automation safety

Bots may classify, check, synchronize, draft, deploy public documentation and package releases. They do **not** automatically merge arbitrary human code or grant external runtime authority.

Dependabot is canonical for dependency update PRs. Do not add Renovate or another dependency bot beside it.

No AI bot is required for deterministic governance. If semantic automation is later justified, it belongs behind the existing sync plane rather than forming a third competing automation system.
