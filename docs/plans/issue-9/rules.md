# Issue #9 — Local agent execution rules

1. Refresh `main` and record the exact SHA before implementation. This plan was prepared from merge commit `458df81f87ae0b328dbc12c2b2829ca3a0ff75d9`; do not assume it remains latest.
2. Read `AGENTS.md`, `blueprint.md`, `README.md`, `docs/AI_HANDOFF.md`, `docs/PROGRESS.md`, `docs/ARCHITECTURE.md`, `docs/COGNITIVE_CORE.md`, `docs/ROADMAP.md`, `docs/terminal-ui.md`, and this complete plan.
3. Audit the actual implementation before changing it. Preserve existing keyboard behavior and ratatui::run lifecycle.
4. Keep changes in the existing Rust package. No new crate, service, server, database, runtime or dependency unless the task proves it is necessary.
5. Treat all data as untrusted. Sanitize error messages and never render secret values.
6. The TUI is read-only in this issue. Do not add shell, arbitrary file writes, tool invocation, model calls, memory mutation, remote egress or evolution promotion.
7. Never invent live tasks, model roles, tools, permission states, metrics or evaluation results. Show unavailable states honestly.
8. Memory content is hidden by default. Do not add write/delete/approval controls.
9. Persist only presentation preferences beneath the executable root. Do not store secrets or duplicate authoritative domain state.
10. Keep task traces content-free and bounded. Use the existing trace reader and validate schema semantics.
11. Do not edit Asmaul Husna glosses, selected enumeration, or theological descriptions. Independent religious/language review is an external dependency.
12. Keep Core Integrity and memory retrieval evaluation offline and deterministic.
13. Update docs and roadmap in the same PR; include test evidence, failure cases and known limitations.
14. Branch prefix: `feat/`; target only `main`; no stacked branches.
15. Close issue #9 only after the PR merges and the resulting main SHA is verified.
