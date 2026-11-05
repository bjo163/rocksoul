# Issue #9 — Cognitive Cockpit completion specification

**Issue:** https://github.com/bjo163/rocksoul/issues/9  
**Base:** `main` after Issue #35 merge, SHA `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`  
**Canonical source:** latest `main`; refresh before implementation.  
**Product boundary:** one portable Rust executable; the TUI is a read-only observer and typed-intent surface over existing services.

## Goal

Complete the existing Ratatui Cognitive Cockpit so it presents truthful, useful runtime state across Overview, Brains, Tasks, Memory, RID, Tools, Diagnostics, and Settings. Reuse existing application services and portable stores. Do not duplicate kernel authority or create new runtime capabilities merely to fill UI screens.

## Audited current implementation

The initial TUI increment is already present in `src/tui.rs`:
- Opt-in `rocksoul tui` command and interactive-terminal guard.
- Ratatui/Crossterm fullscreen lifecycle through `ratatui::run`.
- Eight navigable screens, number keys, Tab/Shift+Tab, command palette, help overlay, refresh, and minimum-size fallback.
- Overview and Brains snapshots based on current model configuration and RID.
- RID screen with the embedded 99-entry catalogue and separate technical definitions.
- Memory summary from the local memory store.
- Tools and task screens explicitly show unavailable/not-persisted state rather than claiming execution.
- Two focused navigation tests.

The initial increment is intentionally incomplete:
- Task screen has no persisted task queue or live task timeline.
- Memory screen is summary-only; no searchable record browser or detail inspector.
- Brains has no specialist registry; do not invent assignments or measured hardware data.
- Tools has no capability registry or invocation surface; do not invent permissions or fake tool rows.
- Diagnostics does not yet present a structured health snapshot or latest evaluation report.
- Settings are not persisted and do not provide a real dark/light/locale preference model.
- The test suite does not yet cover memory/task/diagnostic read models, settings persistence, redaction, or responsive breakpoints.
- `render_memory` still displays `store.v1.json`; the merged memory store is schema v4 and the displayed path must be corrected.

## Scope

1. **Shared read models:** create small typed view models from existing portable services; keep widgets presentation-only.
2. **Task history:** show content-free events from `task_trace` with operation ID, event, timestamp, filename metadata and status. Do not label history as a live task queue.
3. **Memory browser:** list/search/filter local records with kind, claim status, source/provenance, created/expiry metadata and safe detail reveal. Default to metadata-only; never auto-write/delete.
4. **Diagnostics/evaluation:** show actual configuration readiness, RID/memory store health, portable paths, latest Core Integrity report, and honest unavailable fields. Never display credentials or secret values.
5. **Brains/Tools truthfulness:** display only known configured adapters and implemented capabilities. Empty registries must be shown as empty/not implemented, not simulated.
6. **Presentation settings:** persist theme (dark/light/system where supported), density and locale under the portable root. Presentation settings must not change model routing, data egress, permissions or evolution policy.
7. **Accessibility/responsiveness:** keyboard-only navigation, visible focus, semantic status text, minimum-size fallback, bounded lists and readable Arabic/transliteration fallback.
8. **Validation and docs:** tests for read models and state transitions, Windows build/smoke, Linux tests, and updated terminal UI/architecture/handoff documentation.

## Non-goals

- No web, desktop, HTTP API, remote cockpit or multi-package workspace.
- No model/backend calls from widgets.
- No general tool registry implementation, shell execution, plugin/browser/network execution or write-capable tools.
- No autonomous task queue, background workers or live activity fabricated from static state.
- No automatic memory capture, silent memory writes/deletes or automatic context injection.
- No specialist model router or hardware probing unless a separate approved issue defines it.
- No evaluation benchmark integration beyond reading the existing local Core Integrity report; Issue #13 owns the broader evaluation harness.
- No religious review or alteration of Asmaul Husna glosses in this issue; the catalogue remains explicitly draft pending qualified review.

## Required invariants

- Cognitive Kernel and application services remain the only authority for lifecycle, authorization, verification, XP, memory mutations and policy.
- The TUI may read snapshots and emit typed user intents only; it cannot bypass authorization.
- UI state is never authoritative domain state.
- Missing/unavailable values are `n/a`, not zero, guessed, or simulated.
- Secrets are masked and excluded from diagnostics, errors, copy/export and test snapshots.
- Memory content is not shown by default; any reveal is an explicit local user action.
- Destructive/privileged/egress operations are absent until their underlying policy services exist.
- All mutable UI preferences remain under the executable root and preserve across upgrade/re-init.
- TUI exit, error, panic and cancellation restore terminal state using the supported Ratatui lifecycle.
- Existing CLI and REPL behavior remains unchanged.

## Acceptance criteria

- [ ] Existing CLI and REPL regression tests remain green.
- [ ] All eight screens render from real read models; placeholders explicitly identify missing capabilities.
- [ ] Tasks screen displays persisted content-free task trace with honest empty state.
- [ ] Memory screen supports bounded list/search/filter/detail over schema v4 with provenance and safe content reveal.
- [ ] Diagnostics shows actual RID/memory/model config health and latest Core Integrity result, with secret redaction.
- [ ] Brains/Tools never invent specialist assignments, tools, permissions or activity.
- [ ] Dark and light themes work; preferences persist in the portable root and invalid settings fail safely.
- [ ] Locale and density preferences are applied consistently or clearly marked unavailable if not implemented.
- [ ] Keyboard-only navigation, focus, command palette, overlays, and small-terminal fallback are tested.
- [ ] Tests cover read-model mapping, empty/corrupt stores, search/filter, trace corruption, preference migration/validation, secret redaction, and navigation.
- [ ] Linux formatting/Clippy/tests/Core Integrity/memory retrieval pass.
- [ ] Windows build/tests/portable and installer smoke pass.
- [ ] `docs/terminal-ui.md`, README, architecture, roadmap, progress and AI handoff match the delivered UI.
- [ ] Issue #9 is closed only after PR merge and main verification.

## Dependency and follow-up

Issue #13 (AGI-oriented evaluation harness) explicitly depends on #9 and #8. Complete this TUI work before claiming its TUI evaluation/diagnostics deliverable is ready. Issue #8 still requires independent religious/language review; AI-generated review is not a substitute for qualified human review.
