# RockSoul World

RockSoul World is the navigable presentation of the runtime's **actual engineering state**. It is an interaction and information architecture over repositories, systems, evidence, work items, deployments, memory and future world-state primitives; it is not a decorative metaverse simulation.

## World rule

> A world state may look immersive, but it must never claim more than the evidence supports.

The canonical public topology contract is [`../public/world.json`](../public/world.json). The browser World Portal consumes that contract and public GitHub evidence. Ratatui consumes the same topology contract but does not pretend to have live network observations before a Rust Sense/API connector exists.

## Semantic layer

| Engineering object | World term | Authority |
| --- | --- | --- |
| Repository | World Core | Git |
| README | World Entry | repository Markdown |
| Documentation | Codex | `/docs` |
| Wiki | Archive | mirror of `/docs` |
| GitHub Project | World Map | synchronized view of Issues |
| Issue | Quest | GitHub Issues |
| Milestone | Chapter / Region | GitHub Milestones |
| Pull Request | Contribution | GitHub Pull Requests |
| Release | World Build | immutable Git tag + Release |
| Runner | World Node | GitHub Actions runner evidence |
| Automation | World System | `rocksoul-gate` / `rocksoul-sync` |
| Deployment | Portal | deployment provider evidence |
| Vercel | Live World Portal | Vercel deployment |
| Cloudflare | Gateway | explicitly configured private-first gateway |
| TUI | Operator Window | Ratatui runtime |
| Web | World Surface | `public/` |
| Asset | World Resource | `rocksoul-assets` accepted revision |
| Verification | Trust Gate | deterministic checks/evidence |
| Provenance | Evidence Trail | source/ref/hash/timestamp |
| XP | Verified Experience | validated outcome only |
| Level | Capability Progression | progression rules |

Engineering terminology remains visible where it is clearer. The world vocabulary is an interface layer, not a replacement for precise technical language.

## World topology

```text
                         ROCKSOUL WORLD
                              │
              ┌───────────────┼───────────────┐
              │               │               │
          WORLD CORE      WORLD SYSTEMS    WORLD NODES
          rocksoul        cognition         runners
              │           memory            services
              │           knowledge
              │           trust
              │           experience
              │
      ┌───────┼─────────┬──────────┐
      │       │         │          │
    ASSETS    UI       MIND      PORTALS
      │       │         │          │
 resource  grammar   cognition   Pages/Vercel
                                Cloudflare only
                                when justified
```

The map is intentionally topology-first. Geographic/metaverse rendering may be layered later only when the World Graph contains meaningful place/route data.

## Discovery and Fog of War

The shared progression vocabulary is:

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

These states are not interchangeable:

- **UNKNOWN** — no usable evidence.
- **DISCOVERED** — an entity/place is known to exist.
- **OBSERVED** — current or historical evidence has been read.
- **VISITED** — RockSoul intentionally interacted with the place/system.
- **STUDIED** — evidence was analyzed beyond simple presence.
- **VERIFIED** — a defined verification gate passed.
- **TRUSTED** — an explicit trust policy permits this classification; this is never inferred merely from repeated use.

The Web Portal currently uses GitHub public API evidence for repositories, Issues, Releases and repository events. GitHub events are labeled as **GitHub evidence**, not RockSoul runtime telemetry.

## Evidence states

Operational UI uses a separate evidence-state vocabulary:

```text
VERIFIED
OBSERVED
INFERRED
PENDING
BLOCKED
UNKNOWN
```

A component must not silently upgrade a value from `OBSERVED` to `VERIFIED`, or from `PENDING` to `ONLINE`.

## Quests

GitHub Issues are the only canonical public Quest database. The World Portal may present Issues as quests, but it does not copy them into a second task store.

A Quest can surface:

- issue number and objective;
- milestone / Chapter;
- priority and phase labels;
- status;
- evidence links;
- acceptance criteria.

Completion in a game-like interface does not close an Issue unless an authorized engineering action actually closes it.

## Experience and progression

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

Page views, button clicks and cosmetic exploration do not earn XP. Cognitive age remains an evaluation-gated measure and is not chronological age.

## World resources and UI provenance

RockSoul does not create a third design system.

Canonical chain:

```text
rocksoul-assets
      ↓
  @rocksoul/ui
      ↓
RockSoul TUI / World surfaces
```

`rocksoul-assets` is the visual source. `rocksoul-ui` is the implementation grammar. The runtime pins the asset/UI revisions it has accepted in [`../public/brand.env`](../public/brand.env) and [`../public/world.json`](../public/world.json).

A newer asset repository head is not automatically treated as accepted. It must first be reviewed against the UI/runtime contract.

## Portals

### GitHub Pages — Codex

GitHub Pages serves the same static World Surface but enters on the **Codex** view. `/docs` remains canonical; Pages is a navigable presentation.

### Vercel — Live World Portal

Vercel serves the World Surface as the public exploratory portal. A Vercel deployment does not become authoritative runtime state; it reads state/evidence from authoritative sources.

### Cloudflare — Gateway

Cloudflare Tunnel is not a default portal. It is introduced only when a private RockSoul service has a justified public access requirement.

```text
PRIVATE NODE
    ↓
ROCKSOUL SERVICE
    ↓
CLOUDFLARE TUNNEL
    ↓
AUTHORIZED PUBLIC GATEWAY
```

Private-first remains the default.

## World Nodes

A self-hosted runner is a World Node only when actual runner evidence is available. Node status such as `ONLINE`, `READY`, `BUSY` or `OFFLINE` must come from GitHub runner/job evidence or an authorized node registry.

The public GitHub job API can prove which hosted runner executed a job. It cannot, by itself, prove the full inventory or live status of private/self-hosted runners. Until authenticated inventory is available, the World Portal reports those nodes as `UNKNOWN` rather than fabricating a roster.

## No fake metaverse

RockSoul World deliberately avoids:

- fake live node telemetry;
- fake event streams;
- fake trusted states;
- 3D engines without product need;
- duplicate task databases;
- duplicate design systems;
- duplicate cloud databases;
- permanent multi-agent swarms solely for presentation.

The intended result is a **connected, navigable, persistent digital world** whose visual experience follows the architecture instead of hiding it.
