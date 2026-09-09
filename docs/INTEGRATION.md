# Cognitive runtime integration boundary

The `rocksoul-core::integration` module is the provider-neutral boundary for
embedding cognitive services in a host runtime. It exposes versioned requests,
bounded advisory responses, and operator health metadata.

The host runtime remains responsible for lifecycle, persistence, canonical
research ownership, and side effects. The boundary deliberately reports
`advisory_only = true` and `canonical_write_authority = false`; model output,
connector observations, and derived graphs must continue through the existing
verification and promotion gates.

CR-031 is complete only when this contract is consumed by a host integration
without bypassing these constraints and the end-to-end smoke flow is verified.

The `CognitiveSmokeReport` covers the bounded vertical slice in order:
`sense`, `advisory-inference`, `policy`, `evaluation`, `event-replay`, and
`snapshot`. A successful report must explicitly prove that no side effects and
no canonical writes occurred.

`ResourceBudget::bounded()` defines default upper bounds for input, output,
event count, and replay records. Hosts should reject work that exceeds these
limits before invoking a provider or side-effecting operation.

`OperatorHealthSnapshot` is a read-only operator surface. It reports smoke and
certification readiness plus active budgets, and becomes `Degraded` when either
verification input is incomplete. It never grants canonical write authority.
