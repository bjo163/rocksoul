# Template / Fork Guide

RockSoul is designed to be reusable as a GitHub repository template without forcing downstream projects to preserve the RockSoul product name.

Examples:

```text
RockSoul  → StoneSoul
RockSoul  → NovaSoul
RockSoul  → private internal runtime
```

## Template boundary

The **architecture and automation** are reusable. The product identity is not hardwired into the repository platform.

The first file to change is:

```text
public/brand.env
```

Example:

```dotenv
NAME=StoneSoul
SLUG=stonesoul
GENERATION=1
COGNITIVE_AGE=0
LEVEL=1
XP=0
TRUST=0
WORLD=Internet
STATE=Born
PHASE=Phase A / Birth
TAGLINE=Local-first digital cognitive runtime
```

Both the Ratatui header/body and Vercel web mirror use this identity source.

## After creating a repository from the template

1. Change `public/brand.env`.
2. Change package/crate names only if the new project needs different Rust package identifiers.
3. Replace product-name prose in README/docs when the fork is intended to become a different public project.
4. Review `.github/roadmap-issues.json`; keep only phases relevant to the new project.
5. Run `rocksoul-sync` after configuring the GitHub token so labels, milestones, Project and Wiki are rebuilt in the new repository.
6. Connect the new repository to its own Vercel project if the web surface is wanted.
7. Never copy secrets, cloud credentials, private datasets or runtime databases from the source repository.

## What remains generic

- `rocksoul-gate` quality/security/release pattern
- `rocksoul-sync` governance/Project/Wiki/release-draft pattern
- issue forms, PR template and CODEOWNERS
- label/milestone/roadmap JSON as machine-readable governance
- local-first storage rules
- TUI/Web parity contract
- release packaging strategy

## GitHub template setting

`rocksoul-sync` can mark the repository as a GitHub template when `ROCKSOUL_GITHUB_TOKEN` has repository administration permission. This keeps the setting reproducible rather than relying on an undocumented manual toggle.

## Naming rule

Do not introduce a second branding configuration file. Runtime-facing display identity belongs in `public/brand.env`; repository prose may mention the project name naturally, but code should not require mass renaming just to boot.
