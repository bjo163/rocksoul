# Issue #9 — Autonomous task breakdown

**Prepared from main:** `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9` (refresh before starting)  
**Branch:** `feat/tui-cockpit-completion`  
**PR base:** `main` only.

## Phase A — Audit and baseline

- [x] A1. Refresh main; record exact SHA and inspect open PRs/issues for superseding work.
- [x] A2. Read AGENTS, blueprint, terminal UI design, architecture, progress, issue #9 and all four plan files.
- [x] A3. Map all eight screens to real data sources and mark implemented, partial, unavailable or unsafe.
- [x] A4. Audit existing TUI tests, terminal lifecycle, cancellation, secret redaction and Windows behavior.
- [x] A5. Record the existing memory path bug (`store.v1.json` display vs schema v4) and all misleading/static status strings.
- [x] A6. Freeze non-goals: no model calls, tools, memory mutations, arbitrary execution or invented telemetry.

## Incremental implementation notes

The current task branch now includes a bounded persisted task-trace reader and read-only TUI projection, a metadata-only searchable memory browser, the corrected schema-v4 memory path, and a refreshable evaluation-report snapshot. Linux CI has passed for the first implementation slice (111 tests, Clippy, Core Integrity 23/23, memory retrieval precision@2/recall@2 1.000); Windows validation is being repeated against the latest branch head.

The following remain intentionally open: diagnostic parsing/redaction tests for missing configuration and empty registries (E6), persisted preferences and responsive/focus tests (Phase F), and complete TestBackend/Windows validation against the final head (Phase G). This is not Issue #9 completion.

## Phase A — Audit findings recorded

The initial audit confirms the current TUI has eight screens and two navigation tests. The terminal lifecycle is guarded by the interactive-terminal check and `ratatui::run`. Tasks are static rather than a live queue; Memory is a summary only; Brains/Tools lack registries; Diagnostics has no evaluation report; Settings have no persisted preferences. The memory screen still prints the obsolete `store.v1.json` path even though the actual schema is v4. No fake capability or live metric may be added to fill these gaps.

## Phase B — Shared read models and truthful snapshots

- [ ] B1. Add small typed read models for overview, memory metadata, task history, diagnostics and evaluation.
- [ ] B2. Keep all projections pure where possible and source values only from existing services/stores.
- [ ] B3. Bound all lists, text previews, filters and query sizes.
- [ ] B4. Sanitize diagnostics and ensure secrets/credentials never enter UI view models.
- [x] B5. Correct portable memory path to `store.v4.json`; ensure schema version is shown from loaded data.
- [ ] B6. Test missing/corrupt/future-version stores and unavailable values.

## Phase C — Task history / trace screen

- [x] C1. Project persisted JSONL task trace using the existing reader.
- [x] C2. Show event chronology, operation ID, event kind, timestamp and safe target metadata.
- [x] C3. Add bounded selection/detail navigation and a truthful empty state.
- [x] C4. Display malformed/unsupported trace as a sanitized diagnostic; never crash.
- [x] C5. Test valid, empty, malformed, unsupported and large trace fixtures.

## Phase D — Memory browser

- [x] D1. Add bounded memory list with kind, claim status, provenance, approval, created/expiry metadata.
- [x] D2. Add local search/filter by kind, claim status and source; cap query and results.
- [x] D3. Add explicit Enter-to-reveal detail; metadata remains the default view and the shared sensitive-content guard blocks credential/personal-category matches.
- [x] D4. Keep memory content hidden in the TUI, search locally without rendering content, and expose no mutation actions.
- [x] D5. Cover search/filter/empty/expiry/forgetting/supersession in the memory service tests; add TUI filter-cycle, explicit-detail privacy-guard and content-clipping tests.

## Phase E — Diagnostics, evaluation and capability truthfulness

- [x] E1. Extend the existing runtime snapshot with memory-store health and sanitized RID error detail.
- [x] E2. Move the latest Core Integrity summary lookup into the refreshable snapshot; never run the suite during render.
- [x] E3. Show suite/version/pass state and failed check IDs; distinguish not-run from unreadable reports. The current report contract does not expose a tested commit SHA.
- [x] E4. Ensure Brains and Tools show only the known legacy adapter and empty tool registry; unknown specialist roles remain unassigned.
- [ ] E5. Remove stale/fake status labels; use explicit “not implemented”, “not configured”, “not run” or “unavailable”.
- [ ] E6. Test report parsing, invalid JSON, redaction, missing config and empty registries.

## Phase F — Preferences, accessibility and responsive behavior

- [ ] F1. Add versioned portable TUI preferences for dark/light theme, density and locale.
- [ ] F2. Apply semantic theme tokens consistently to every screen and overlay.
- [ ] F3. Preserve safe defaults and never overwrite unknown future preference schemas.
- [ ] F4. Add visible focus state and keyboard-only access to all new controls.
- [ ] F5. Test 80×24, narrow terminal fallback, resize, overlays and focus transitions.
- [ ] F6. Ensure preferences do not modify model routing, egress, secrets or security policy.

## Phase G — Validation and handoff

- [ ] G1. Add TestBackend coverage for all read models, navigation and screen empty/error states.
- [ ] G2. Run fmt, Clippy, all tests, Core Integrity and memory retrieval evaluation.
- [ ] G3. Verify Windows build/tests/portable smoke/installer smoke.
- [ ] G4. Verify no model/backend calls, tool invocations or memory mutations originate from widgets.
- [ ] G5. Update terminal-ui.md, README, architecture, roadmap, progress and AI handoff.
- [ ] G6. Inspect complete diff and compare it with Issue #9 acceptance criteria.
- [ ] G7. Open PR to main; merge only after CI passes.
- [ ] G8. Verify main SHA, close issue #9 with evidence, and update handoff with the next dependency-safe task.

## Completion output

Record exact main base/head/merge SHA, PR URL, CI run IDs, test count, Core Integrity and retrieval report results, Windows smoke results, known limitations, issue closure, and next task. Do not mark issue #9 complete merely because the UI compiles.
