# Project Management

RockSoul uses **issues + milestones as the canonical project model**. GitHub Project v2 is a visualization/planning layer over those canonical issues, not an independent source of truth.

## Why this split

Issues and milestones are repository-native, easy to automate with `GITHUB_TOKEN`, and remain useful even if a Project view is deleted or reconfigured.

GitHub Project v2 is useful for board/table/roadmap views, but user-owned Projects require a token with the `project` scope. The default Actions token is not assumed to have that authority.

## Canonical phases

```text
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

The governance sync workflow creates a milestone and canonical tracking issue for each phase.

## Optional GitHub Project v2

`.github/workflows/project-sync.yml` can create/link **RockSoul Roadmap** and add open repository issues to it.

It requires a repository Actions secret:

```text
ROCKSOUL_PROJECT_TOKEN
```

The token must belong to an account allowed to manage the user-owned project and include GitHub's `project` scope.

The workflow is intentionally manual (`workflow_dispatch`) so a project-scoped token is never exercised unexpectedly.

## Recommended Project views

Once synced:

- **Board:** Status
- **Roadmap:** Phase / milestone
- **Table:** priority, labels, milestone, assignee
- **Current:** open issues in the active phase

Project fields beyond GitHub's built-in Status should only be added when they represent information not already captured reliably by labels/milestones.
