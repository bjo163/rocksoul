# Project Management

RockSoul uses **Issues + Milestones + Labels as the canonical project model**. The public GitHub Project named **`rocksoul`** is a synchronized visualization layer, not an independent database.

## Public Project layout

`rocksoul-sync` creates or reuses the user-owned Project `rocksoul`, sets it to **PUBLIC**, links this repository, and keeps open issues present.

The intended views are:

### Kanban

Board view for execution flow. GitHub's built-in Status field remains the visual workflow column; labels provide orthogonal information such as priority, type, area and phase.

Recommended status flow:

```text
Todo → In Progress → Done
```

Do not encode the same state again in several custom fields.

### Roadmap

Roadmap view for milestones/phases. Foundation and Phase A–M milestones communicate the larger sequence while the custom `Phase` Project field provides compact grouping/filtering.

### Backlog

Table view for dense triage and planning across all open issues.

Useful columns/filters include:

- Status
- Priority
- Phase
- Milestone
- labels
- assignee

## Canonical fields

### Labels

Labels remain the richest machine-readable taxonomy:

```text
type:*
area:*
priority:p0..p3
status:*
phase:*
release:*
```

### Milestones

Milestones represent Foundation and major cognitive phases:

```text
Foundation
A  Birth
B  World
C  Memory
D  Nano
E  Skills
F  Cognition
G  Guardian
H  Life
I  Cortex
J  Shura
K  Autonomous Research
L  Collective Intelligence
M  ASI Research Gate
```

### Project custom fields

Only two additional single-select fields are created initially:

- **Priority:** P0, P1, P2, P3
- **Phase:** Foundation, A–M

Those fields are synchronized from issue labels. We deliberately do not create redundant Type/Area/Milestone custom fields because GitHub already exposes labels and milestones.

## Synchronization

The synchronization contract is one-way for canonical metadata:

```text
.github/labels.json ───────────────→ GitHub Labels
.github/milestones.json ──────────→ GitHub Milestones
.github/roadmap-issues.json ──────→ canonical Issues
Issue labels + milestone ─────────→ public Project fields/views
/docs ────────────────────────────→ GitHub Wiki
```

Issues remain usable even if the Project is removed, recreated or temporarily inaccessible.

## Activation token

Basic Issue/Label/Milestone sync works with GitHub Actions' normal repository token.

Public Project, Wiki push and repository-admin settings use one optional secret:

```text
ROCKSOUL_GITHUB_TOKEN
```

The token should belong to the repository owner and include the GitHub Projects `project` scope plus sufficient public-repository administration/Wiki access for the requested synchronization.

Using one privileged sync token is preferable to separate Project/Wiki/repository tokens because it reduces secret and automation overlap.

## Tags and releases

Git tags are not a planning database. They represent immutable public versions after a deliberate age-release action. See `DEPLOYMENT.md` and `GOVERNANCE.md`.
