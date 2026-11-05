# Rocksoul Terminal UI — Cognitive Cockpit

**Status:** The opt-in Ratatui Cognitive Cockpit is implemented and merged to `main` via PR #65 (`6474cd130bd8146f03dcacb757d9c341d4b1161a`). Exact-head Linux and Windows CI #42071047960 passed. It includes bounded persisted task history, event selection/detail, searchable/filterable memory with guarded explicit reveal, snapshot diagnostics/evaluation, versioned portable preferences, semantic dark/light palettes, density limits, and English/Indonesian navigation.  
**Product:** Rocksoul local-first cognitive runtime  
**Interface:** Native terminal application

## 1. Product narrative

Rocksoul starts as one executable and one CLI. Its terminal interface should feel like the cockpit of a small, capable machine: alive when real work is happening, quiet when idle, direct when a human needs to intervene.

The full-screen experience is named **Rocksoul Cognitive Cockpit**.

The cockpit is not a chatbot skin, a decorative system monitor, or a second application architecture. It is an opt-in terminal presentation over the same Rocksoul executable and local state used by the CLI. A future service/API layer is a roadmap possibility, not a current interface.

**Experience statement:** Local intelligence. Observable decisions. Human control.

The visual direction is dark industrial / terminal HUD: charcoal surfaces, disciplined grid lines, warm signal amber, restrained phosphor accents, and compact status typography. It should feel distinctive and premium without sacrificing speed, legibility, or the low-resource character of a single Rust executable.

## 2. Product boundaries

The current executable exposes CLI commands, an interactive REPL, and an opt-in TUI. These are entry points in one process/runtime, not separate services:

| Entry point | Command | Purpose |
|---|---|---|
| Scriptable CLI | rocksoul status, rocksoul rid stats, etc. | Stable commands, automation, piping, CI |
| Interactive REPL | rocksoul | Direct conversational interaction |
| Full-screen cockpit | rocksoul tui | Live inspection, navigation, and authorized controls |

Rules:
- TUI is opt-in. Do not automatically enter full-screen mode when stdin/stdout are piped, redirected, or non-interactive.
- CLI output remains stable and machine-friendly; never replace it with TUI output.
- The cockpit and REPL must not own separate model, memory, task, or identity state.
- The UI does not decide policy, award XP, mutate memory directly, select unauthorized tools, or promote self-evolution candidates. It sends typed intents to application services; the kernel decides.
- Local-first behavior is fixed. The UI may show remote provider availability, but must not present local / hybrid / online as competing product modes.

## 3. Visual language

### Mood
**Industrial console × cognitive instrument panel × restrained cyberpunk.**

Use near-black / graphite backgrounds, warm bone text, amber for active focus, muted teal or phosphor green for healthy states, and red only for failures or security warnings. Use thin separators, compact metadata, and consistent spacing.

Avoid giant ASCII logos, decorative noise, fake scanlines, excessive borders, emoji-heavy status rows, and constant blinking. Avoid RGB-only styling that fails in ordinary Windows Terminal or SSH sessions. Never show fake telemetry or simulated progress.

### Semantic status system
Each status has text plus optional color:
- READY — healthy and available.
- WORKING — an operation is genuinely active.
- WAITING — waiting on model, user approval, or external response.
- DEGRADED — usable with a known limitation.
- BLOCKED — policy or permission prevents the action.
- FAILED — an operation failed.
- OFFLINE — local service or backend is unavailable.

Do not use color as the only status signal.

### Typography and density
Prioritize readable monospace labels, aligned values, compact but non-cramped rows, and stable high-frequency information. Long text wraps in a detail pane, never across adjacent panels.

## 4. Global cockpit frame

The full-screen frame has three layers:
1. Top rail: Rocksoul identity, runtime state, active brain/backend, local/remote connection state, current time.
2. Workspace: current screen with a main work area and optional inspector/context pane.
3. Command rail: contextual action hint, command-palette hint, active selection, global key hints.

Illustrative layout:

    ┌ ROCKSOUL / COGNITIVE COCKPIT ───────── LOCAL RUNTIME: READY ┐
    │ CORE READY   BRAIN LANGUAGE/ACTION   MODEL local-model RID L2 │
    ├───────────────────────────────────────────────────────────────┤
    │ 01 OVERVIEW  02 BRAINS  03 TASKS  04 MEMORY  05 RID  06 TOOLS │
    ├──────────────────────────────────────────┬────────────────────┤
    │ CURRENT WORK                             │ SYSTEM INSPECTOR   │
    │ Turn / task timeline                     │ Active backend     │
    │ Recent verified events                   │ Model readiness    │
    │ Resource and budget snapshot             │ Permissions        │
    │                                          │ Recent events      │
    ├──────────────────────────────────────────┴────────────────────┤
    │ > Command / action                                 Ctrl+K help │
    └───────────────────────────────────────────────────────────────┘

At widths where the inspector would become unreadable, collapse it into a selectable detail view rather than shrinking every panel.

## 5. Navigation and interaction

Keyboard-first, with optional mouse support. All core workflows must be possible without a mouse.

| Key | Action |
|---|---|
| 1..8 | Open a numbered primary screen |
| Tab / Shift+Tab | Move focus between regions |
| Up / Down | Move within the focused list |
| Left / Right | Change tab or horizontal selection |
| Enter | Open selected item or confirm safe action |
| Esc | Close overlay, clear filter, or return |
| Ctrl+K | Open command palette |
| / | Search the current screen |
| ? | Show contextual key help |
| r | Refresh the current read model |
| q | Exit cockpit |

Command palette:
- Search registered commands and navigation targets.
- Show required arguments and permission level before execution.
- Never execute a destructive or external side-effect action merely because a fuzzy match was selected.
- Require separate confirmation for irreversible, privileged, or data-egress operations.

## 6. Primary screens

### 6.1 Overview / Command Deck
Answer: Is the kernel ready? Which brain role is serving the current task? What task/turn is active? What is the model/backend readiness? What needs attention? What is the RID state?

Show only real runtime measurements: task phase, request latency, tokens where available, resource usage where available, and budget consumption. If a metric is unavailable, show n/a; never synthesize a number.

### 6.2 Brains / Runtime
Inspect automatically assigned roles: Reflex, Language / Action, Reasoning / Planning, Decision, Perception, and Embedding / Reranking. Each role shows assigned backend/model, local availability, capability fit, readiness, last health check, and measured latency where available. Explain unavailable or degraded states. Users do not choose local/hybrid/online as a top-level mode.

### 6.3 Tasks / Turn Monitor
Show the active task and Cognitive Kernel lifecycle: Planning, Executing, Verifying, Completed, Failed, and Cancelled. Provide chronological event trace, current action, retry count/limit, remaining budget, wait reason, and verification result. A proposed action must be visually distinct from an authorized and executed action.

### 6.4 Memory / Experience
Provide searchable, filterable, provenance-aware inspection of working, episodic, semantic, procedural/skills, self-model, and temporal memory. Default view is read-only. Show source, timestamp, confidence, retention state, and contradictions. Forget/delete/export actions use the same policy service as every interface and require explicit confirmation where destructive.

### 6.5 RID / Ruh Information
RID is a first-class cockpit screen.

Header metrics: instance age, level, total XP, XP progress to next level, and rated/unrated stat count.

The 99-stat browser:
- Group by the nine technical domains.
- Filter all / rated / unrated / needs review.
- Search by stable stat key, technical label, transliteration, or catalogue ID.
- Detail pane shows Arabic display label, transliteration, Indonesian/English draft gloss, technical capability definition, evaluation protocol, rubric version, confidence, sample count, evidence references, and last evaluation time.

**Semantic separation is mandatory:** Asmaul Husna is a respectful naming/inspiration layer. The measured value is an engineering capability of Rocksoul, never a measurement or possession of a divine attribute. Display catalogue provenance and review status. Unrated is not zero.

### 6.6 Tools / Permissions
Show registered capabilities, read/write/execute permissions, approval requirements, sandbox boundary, and recent invocation results. Never silently elevate permissions or expose secrets. Credentials are always masked.

### 6.7 Diagnostics / Events
Show structured events, warnings, failures, and health checks. Support severity, component, time-range, and task filters. Redact secrets and configured sensitive payload fields. Allow copy/export of sanitized diagnostics.

### 6.8 Settings
Presentation settings: dark/light/auto theme, compact/comfortable density, Indonesian/English language, keymap, refresh behavior, reduced motion, and log verbosity. Model routing, remote data egress, tool permissions, and evolution policy remain governed by policy/configuration services, not cosmetic UI preferences.

## 7. Responsive behavior

| Terminal size | Layout |
|---|---|
| 120×36 and above | Full cockpit with main pane and inspector |
| 80×24 to 119×35 | Compact cockpit; inspector becomes a tab/detail drawer |
| Below 80×24 | Minimal navigation + focused single-pane view, or a clear size warning with safe exit |

Respect resize events, avoid horizontal scrolling for primary information, and use vertical scrolling for long logs and the 99-stat list. Arabic glyph rendering may depend on terminal font and shaping support; transliteration must remain a readable fallback. Right-to-left text must not corrupt table layout.

## 8. Runtime architecture

### Recommended stack
- Ratatui 0.30.x for widgets, layout, buffered rendering, and test backend.
- Crossterm as the initial cross-platform terminal backend for Windows, Linux, and macOS.
- Existing Tokio runtime for asynchronous application services.
- Typed UI view models populated from application-service snapshots and runtime events.

Ratatui uses immediate rendering with intermediate buffers. Its current documentation recommends ratatui::run for the common full-screen setup because terminal modes are restored on normal exit and panic; TestBackend supports deterministic widget tests. Pin the exact compatible crate version in Cargo.lock when implementation begins.

### Boundaries

    Terminal Events ──┐
    Runtime Events ───┼──> TUI State / View Models ──> Ratatui Renderer
    User Intents ─────┘              │
                                     v
                             Application Services
                                     │
                                     v
                              Cognitive Kernel
                                     │
                  ┌──────────────────┼──────────────────┐
                  v                  v                  v
               Models             Memory             Tools

- Widgets render data and emit typed user intents.
- Application services perform validation and authorization.
- Cognitive Kernel remains authoritative for lifecycle, budgets, permissions, execution, verification, XP authorization, and promotion.
- The TUI must never call the model backend directly.
- The TUI must not maintain a second copy of authoritative task, memory, or RID state.
- Keep UI read models separate from persistent domain models.

### Event loop and rendering
- Bridge terminal input into the async application without blocking model or kernel work.
- Use event-driven redraws; a low-frequency tick may be used only for genuinely time-dependent presentation.
- Render from immutable snapshots or state owned by the UI controller.
- Debounce resize and search events where appropriate.
- Do not continuously repaint or animate when no visible state changes.
- Ensure terminal restoration on normal exit, error, panic, and cancellation.

## 9. Accessibility, safety, and reliability

- Complete keyboard navigation and visible focus state.
- Semantic text statuses, not color-only meaning.
- High-contrast theme and reduced-motion option.
- Safe behavior when Unicode, color, mouse, or alternate-screen support is missing.
- Clear exit path that restores terminal settings.
- Mask API keys, tokens, and secrets in all screens and exports.
- Confirm destructive, privileged, remote-egress, and evolution-promotion operations.
- Do not let display state imply an operation succeeded; only verified runtime events may do so.

## 10. Performance goals

- Low idle CPU usage; redraw on state changes rather than a busy loop.
- No network or model call required to open or navigate the cockpit.
- No heavy image rendering, GPU dependency, or background polling loop by default.
- Virtualized or bounded rendering for long logs and 99-stat views.
- Fast startup using already initialized runtime state.
- Memory use should remain small relative to the model runtime; TUI dependencies must not pull in heavyweight graphics stacks.

## 11. Implementation sequence

### Implemented in the initial cockpit increment
- Opt-in `rocksoul tui` command and interactive-terminal guard.
- Ratatui/Crossterm fullscreen lifecycle using `ratatui::run`.
- Eight navigable screens, keyboard controls, command palette and help overlay.
- Real local configuration/model/RID snapshot and 99-stat list/detail.
- Honest unavailable-state labels for task queue, memory, specialist routing and tools.
- Responsive minimum-size warning and read-only inspector.

The initial increment does not yet implement persistence for UI preferences, real task/event streaming, memory browsing, model registry, tool actions, or evaluation dashboards.

## 11. Implementation sequence

1. TUI foundation: opt-in rocksoul tui, terminal lifecycle guard, panic-safe restore, event bridge, non-interactive detection.
2. Design system: theme tokens, semantic statuses, layout primitives, keyboard focus, resize handling, and test backend.
3. Overview: read-only runtime snapshot and event summary using existing application services.
4. Task monitor: Cognitive turn state, timeline, budgets, and wait/error status.
5. RID screen: 99-stat catalogue, evidence, age, level, XP, and unrated state.
6. Brains / Memory / Tools: read-only inspectors with controlled action intents.
7. Command palette and polish: search, confirmation flow, accessibility, Windows Terminal validation, and documentation.

## 11.1 Merged delivery — PR #65

The following behaviors are implemented on `main` and validated by CI #42071047960:

- Task Monitor reads the existing `logs/task-trace.v1.jsonl` records through a bounded reader (maximum 1 MiB and 12 newest events).
- Events are selectable with Up/Down; the selected event shows only event type, operation ID, target filename and timestamp. Trace detail/body is never rendered.
- Empty traces show a truthful empty state. Malformed, unsupported or inaccessible traces show a sanitized unavailable state.
- Memory browser lists bounded metadata (kind, claim status, source, approval, creation and expiry); Enter explicitly reveals selected content only after the shared sensitive-content guard approves it.
- Memory search is local. Kind and claim filters are keyboard-controlled; exact source filtering uses `source:<value>`.
- Memory path display uses `store.v4.json`.
- Latest Core Integrity report is read into the refreshable snapshot, not loaded during rendering. Missing and unreadable reports are distinguished; failed check IDs are shown.

The PR does not add a live task queue, tool registry, model registry, memory mutation actions, automatic memory capture, or memory context injection. These remain unavailable or explicitly deferred.

Validation evidence and completion status are tracked in [the Issue #9 execution plan](plans/issue-9/task.md), [progress snapshot](PROGRESS.md), and [Cognitive Cockpit architecture](COGNITIVE_COCKPIT_ARCHITECTURE.md). PR #65 is merged and Issue #9 is closed.

## 12. Acceptance criteria

- Existing CLI and REPL behavior remain unchanged.
- TUI launches only when requested and refuses unsafe full-screen startup on redirected/non-interactive streams.
- TUI works in Windows Terminal and common ANSI terminals on Linux/macOS.
- Terminal state restores after exit, error, panic, and cancellation.
- All displayed data originates from actual runtime snapshots/events; unavailable values show n/a.
- UI never bypasses kernel authorization or mutates authoritative state directly.
- Keyboard-only navigation covers every primary screen and action.
- Layout remains usable at 80×24 and degrades safely below that size.
- RID shows exactly 99 stable stat slots with evidence-based values and explicitly separate Asmaul Husna display labels.
- Tests cover view models, navigation, state transitions, resize, error paths, and key security redaction.

## 13. References

- Ratatui crate and current API: https://docs.rs/ratatui/latest/ratatui/
- Ratatui terminal lifecycle / ratatui::run: https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html
- Ratatui backends and Crossterm: https://ratatui.rs/concepts/backends/
