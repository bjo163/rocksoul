# Repository Governance

RockSoul uses machine-readable repository governance so GitHub surfaces can be rebuilt instead of maintained by hand.

## Sources of truth

| Concern | Source of truth |
| --- | --- |
| release code | `main` |
| integration code | `dev` |
| feature work | `feature/*` |
| architecture | `docs/ARCHITECTURE.md` |
| phase intent | `docs/ROADMAP.md` |
| executable work | GitHub Issues |
| phase grouping | GitHub Milestones |
| label taxonomy | `.github/labels.json` |
| canonical roadmap/foundation issues | `.github/roadmap-issues.json` |
| automation | `gate.yml`, `sync.yml`, Dependabot |
| TUI/Web display identity | `public/brand.env` |

## Branch policy

The intended branch namespace is deliberately small:

```text
main
  production/release source

dev
  integration branch

feature/*
  short-lived human work

dependabot/*
  short-lived GitHub-native dependency bot exception
```

Do not create `release/*`, `hotfix/*`, `chore/*`, `docs/*`, or long-lived experiment branches. Use `feature/<purpose>` and labels to describe the work type.

`rocksoul-gate` rejects pull requests from human branch names outside `dev` and `feature/*`.

Two legacy `chore/*` branches may still exist from the repository bootstrap. They should be deleted once no longer referenced; the policy above applies to all new work.

## `main` protection

`main` should be protected with a GitHub ruleset requiring:

- pull request before merge;
- required `rocksoul-gate` checks;
- no force push;
- no branch deletion;
- squash merge as the normal history strategy.

Branch protection is an actual GitHub admin setting. Automation must not claim it exists until GitHub reports the branch/ruleset protected.

## Merge strategy

- **Squash merge:** enabled and preferred.
- **Merge commit:** disabled once repository-admin sync is active.
- **Rebase merge:** disabled once repository-admin sync is active.
- **Delete branch on merge:** enabled once repository-admin sync is active.

This keeps a public project history easy to read and makes automatic feature-branch cleanup possible.

## Labels

Labels are namespaced:

- `type:*` — bug, feature, docs, architecture, maintenance, security, research;
- `area:*` — core, life, TUI, Web, World, Memory, Brain, Senses, Skills, Cognition, Guardian, Storage, CI;
- `priority:p0..p3` — urgency;
- `status:*` — ready, needs-design, blocked;
- `phase:*` — Phase A–M;
- `release:*` — release-note intent;
- meta labels such as `dependencies`, `breaking-change`, `stale`, `roadmap`.

Labels are synchronized by `rocksoul-sync`, not maintained as an undocumented UI-only taxonomy.

## Issues and milestones

Each canonical work item should have:

- a single clear goal;
- scope and non-goals;
- measurable acceptance criteria;
- dependencies/blockers when known;
- one phase/Foundation milestone;
- labels for type, area, priority and phase/status.

The public Project is a visualization over these issues; moving a Project card must not invent a second project model that contradicts issue labels or milestones.

## Pull requests

PRs should be coherent rather than artificially tiny. One PR may update implementation, docs, tests and automation when they represent the same change.

Path labeling is automatic. `rocksoul-gate` is the only deterministic merge-quality gate.

## Age-aligned releases

RockSoul does not create a release for every merge.

When a meaningful public build is ready, manually run `rocksoul-sync` with `publish_age_release=true`. It creates the next:

```text
v0.<cognitive-age>.<revision>
```

The tag starts `rocksoul-gate`, which validates the same code and publishes Linux, Windows and macOS binaries to GitHub Releases.

Cognitive age is an evaluated project/runtime concept. The release tag mirrors that state; the version itself is not evidence of intelligence or maturity.

No release branches are required.

## Template policy

The repository is intended to be reusable as a GitHub template. Product identity belongs in `public/brand.env`; automation and architecture should not require mass-copy edits for a derivative such as StoneSoul.

See `docs/TEMPLATE.md`.

## Automation safety

Bots may classify, check, synchronize, draft and package. They do **not** automatically merge arbitrary human code and do not grant external runtime authority.

Dependabot is canonical for dependency update PRs. Do not add Renovate or another dependency bot beside it.
