# Deployment

RockSoul exposes one shared world identity through several deliberately different surfaces:

- **Ratatui — Operator Window:** local/native operator interface.
- **GitHub Pages — Codex:** public documentation/world-entry surface.
- **Vercel — Live World Portal:** public exploratory World surface.

All three use canonical identity/topology from `public/brand.env` and `public/world.json`. The Web surface intentionally remains framework-free in the current phase; it is static HTML/CSS/JS plus public GitHub evidence.

## Local state

The repository has **not been cloned locally as part of the current repository-platform work**. GitHub writes plus GitHub-hosted CI are sufficient for this pass.

If a future task truly requires a local clone, report that dependency before creating it.

## Ratatui Operator Window

```bash
cargo run -p rocksoul-tui
```

Release binary:

```bash
cargo build --release -p rocksoul-tui
```

The current TUI navigation is:

```text
WORLD  MAP  QUESTS  CODEX  SYSTEM
```

It consumes the same `brand.env` and `world.json` topology as Web. It does **not** fetch GitHub/network state yet; places that require live network evidence remain explicit `UNKNOWN/PENDING` in the native UI until a Rust Sense/API connector exists.

## Local Web surface

Serve `public/` with any static server:

```bash
python -m http.server 3000 -d public
```

Open `http://localhost:3000`.

This command is documentation only; it is not evidence that a local server was started during the repository-platform pass.

## GitHub Pages — Codex

Pages and Vercel serve the same compact static artifact but enter with different intent:

- on `*.github.io`, `app.js` opens the **CODEX** view by default;
- elsewhere, including Vercel, it opens the **WORLD** view by default.

This avoids creating a second documentation application.

`rocksoul-sync` owns Pages as a sub-job of the existing synchronization plane:

1. privileged repository sync enables Pages with `build_type=workflow` when `ROCKSOUL_GITHUB_TOKEN` has the required Pages/Admin permission;
2. the Pages job checks whether Pages is actually enabled;
3. `actions/configure-pages@v5`, `actions/upload-pages-artifact@v4` and `actions/deploy-pages@v4` deploy `public/`;
4. when Pages is not enabled, the job reports that state rather than pretending deployment succeeded.

Expected public URL after verified activation:

```text
https://bjo163.github.io/rocksoul/
```

Until the Pages API reports a configured site and a deployment succeeds, its status is **PENDING**.

## Vercel — Live World Portal

The repository contains `vercel.json` with `public/` as the output directory and no application build command.

Canonical target:

```text
GitHub repository: bjo163/rocksoul
Vercel project:    rocksoul
Production source: main
Preview source:    feature/*
```

`dev` is the integration branch and may receive a stable preview environment when useful.

An existing Vercel project linked to another RockSoul repository is **not** a substitute for this canonical deployment. The project is only considered connected after Vercel reports that `bjo163/rocksoul` is its Git source and an actual deployment can be inspected.

## World Portal state sources

The current static portal combines:

```text
public/brand.env     identity / Age / Level / XP / Trust
public/world.json    topology / evidence vocabulary / accepted asset refs
GitHub public API    repositories / Issues / Releases / repo events
```

GitHub repository events are shown as **GitHub evidence**, not as RockSoul runtime telemetry.

Future runtime events should use a separate authenticated API/state contract rather than overloading GitHub activity.

## Surface parity contract

TUI and Web keep these shared concepts aligned:

```text
Name
Generation
Cognitive Age
Level
XP
Trust
World
State
Phase
Navigation
World topology
Evidence vocabulary
Accepted asset/UI provenance
```

The rendering can differ because terminal and browser affordances differ. The state meaning must not.

## Cloudflare — Gateway

Cloudflare Tunnel is **not** required for the current public static surfaces: GitHub Pages and Vercel already provide public delivery.

Use Tunnel only when an actual private service must be reached through an authorized public gateway:

```text
PRIVATE NODE
  ↓
ROCKSOUL SERVICE
  ↓
CLOUDFLARE TUNNEL
  ↓
AUTHORIZED PUBLIC GATEWAY
```

Default: **PRIVATE FIRST**.

Cloudflare R2 remains an optional future object-storage target for large immutable/public objects; architecture documentation is not evidence that a bucket has been provisioned.

## Age-aligned releases

Release publication is intentionally lightweight and opt-in.

Two supported triggers:

1. merge a PR carrying `release:ready`; or
2. manually run `rocksoul-sync` with `publish_age_release=true`.

The sync workflow then:

1. reads `COGNITIVE_AGE` from `public/brand.env`;
2. calculates the next `v0.<age>.<revision>` tag;
3. pushes the tag;
4. explicitly dispatches `rocksoul-gate` on that exact tag because ordinary `GITHUB_TOKEN` pushes do not recursively trigger another workflow;
5. the gate repeats policy, Rust, portal/docs, security and CodeQL checks;
6. only then it creates the GitHub Release and packages binaries.

Release assets:

- Linux x86_64 — `rocksoul-linux-x86_64.tar.gz`
- Windows x86_64 — `rocksoul-windows-x86_64.zip`, containing `rocksoul.exe`
- macOS arm64 — `rocksoul-macos-arm64.tar.gz`

Example at Cognitive Age `0`:

```text
v0.0.1
v0.0.2
v0.0.3
```

When Cognitive Age advances to `1`, the revision line begins at:

```text
v0.1.1
```

This is distribution versioning aligned to evaluated age. It is not evidence of AGI/ASI or chronological age.

## Deployment authority

- GitHub `main` is the releasable code source.
- `dev` is integration; `feature/*` is short-lived work.
- GitHub Pages and Vercel are presentation portals, never source databases.
- GitHub Releases are the canonical binary distribution surface.
- Cloudflare R2 may mirror large immutable artifacts only when actually configured.
- Public status is reported as `VERIFIED/OBSERVED/INFERRED/PENDING/BLOCKED/UNKNOWN`; never infer `ONLINE` from configuration alone.
