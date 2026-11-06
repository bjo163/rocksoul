# Independent Effect Verifier Contract

Status: design contract; not implemented.
Scope: Issue #13 agent-trajectory evaluation.
Last reviewed: 2026-11-06.

This document defines the trust boundary between an untrusted model/guest trajectory and the component allowed to attest that a task's requested effect actually occurred. It is deliberately a protocol contract, not a claim that Rocksoul currently has an independent verifier, signed attestations, or a production sandbox.

## 1. Security objective

A trajectory may be marked successful only when a verifier outside the model/guest trust domain observes the required effect through an independently controlled observation channel and binds that observation to the exact task attempt.

The model, planner, guest runtime, tool output, trace text, and model-authored summaries are untrusted inputs. None may mint verifier identity, choose its own trust root, assert an observed effect, or convert a self-report into success.

## 2. Trust domains

| Component | Trust | Allowed responsibility | Must not do |
|---|---|---|---|
| Model / planner | Untrusted | Propose steps and tool arguments | Author verification or success |
| Guest / sandboxed executor | Constrained, untrusted for truth claims | Execute allowlisted operations | Access verifier keys or policy store |
| Host policy broker | Trusted computing base | Authorize task-scoped capabilities and budgets | Trust guest-supplied permissions |
| Effect adapter | Trusted only for its narrowly reviewed observation method | Read the actual target effect | Reuse guest output as observed state |
| Independent verifier | Trusted only for configured rubric and adapter | Evaluate observed effect against task predicate | Accept model-authored evidence as ground truth |
| Report aggregator | Trusted policy component | Validate attestation, enforce vetoes and aggregate evidence | Upgrade missing/invalid evidence to pass |
| Persistence | Integrity-protected host boundary | Store immutable run and evidence records | Silently rewrite prior attestations |

Trust is scoped. A verifier trusted for a fixture key/value store is not automatically trusted for filesystem, browser, network, memory, or operating-system effects.

## 3. Task-bound challenge envelope

Before execution, the host creates an immutable task-attempt envelope containing:

- schema version and protocol version;
- globally unique run ID and task-attempt ID;
- cryptographically random, single-use challenge nonce (minimum 128 bits);
- Rocksoul source commit or explicit `unknown`;
- suite, task, dataset and rubric identifiers plus immutable versions/digests;
- model/provider/runtime identifiers where applicable;
- requested effect predicate and its canonical digest;
- allowed tool IDs, capability grants and resource ceilings;
- start time and expiry/deadline;
- verifier policy ID and permitted verifier identities.

The host, not the model, creates and persists this envelope before the first tool call. The guest receives only the minimum task-scoped fields needed to execute. It never receives verifier signing keys, trust-store write access, or unrelated secrets.

## 4. Observation and attestation

A verifier adapter must independently observe the target system after the action. It must not treat any of the following as sufficient evidence:

- model text saying the task is complete;
- guest stdout/stderr or tool result text alone;
- a guest-provided hash without an independently read source;
- a trace event emitted only by the component being evaluated;
- an evidence URI that the verifier cannot resolve and inspect.

A successful attestation binds at least:

- protocol/schema version;
- run ID, task-attempt ID and challenge nonce;
- task-predicate digest and rubric version;
- verifier identity, adapter identity and adapter version;
- observed target identity and observation method;
- observed result digest (avoid raw sensitive values);
- evidence references resolvable by the host;
- pass/fail/indeterminate decision and reason code;
- confidence with an explicitly defined calibration basis (never an arbitrary model confidence);
- observation timestamp and verifier expiry;
- signature or equivalent authenticated integrity proof from a host-managed verifier identity.

The host validates the attestation against a preconfigured trust store and the original task envelope. The guest cannot register verifier identities, rotate trust roots, or change verifier policy during a run.

## 5. Replay, substitution and freshness controls

- Nonces are single-use and scoped to one task attempt.
- Duplicate attestation IDs, reused nonces, wrong run/task IDs, mismatched predicate digests, stale timestamps and expired attestations fail closed.
- A verifier identity must be authorized for the exact adapter, effect class and rubric.
- Evidence references must resolve to immutable, content-addressed evidence or a host-controlled append-only record.
- A retry creates a new task-attempt ID and nonce. Evidence from an earlier attempt cannot silently satisfy the retry.
- A changed task predicate, dataset, rubric, model configuration or execution policy creates a distinct comparable-run identity.
- An unavailable verifier returns `indeterminate`, never `pass`.

## 6. Decision semantics

Use three-valued verification:

- `pass`: valid trusted attestation proves the required effect.
- `fail`: trusted observation proves the required effect did not occur or a required invariant was violated.
- `indeterminate`: observation unavailable, incomplete, ambiguous, expired or not trusted.

Only `pass` may satisfy a required success predicate. `indeterminate` is not a partial pass. Safety, security, privacy, permission, sandbox-escape and secret-exposure violations are hard vetoes and cannot be averaged away by task performance.

A verifier may report multiple separately observable predicates. Each predicate needs its own rubric and evidence reference where the RID overlap registry requires distinct outcomes. One undifferentiated outcome must not be counted as evidence for multiple adjacent RID dimensions.

## 7. Isolation requirements

- Verifier process and policy broker run outside the guest process and guest-controlled address space.
- Verifier credentials and signing material are unavailable to the guest and are not inherited into its environment or handles.
- The guest cannot write verifier storage, trust configuration, host reports, or task-envelope records.
- Verifier communication is a narrow authenticated IPC protocol with bounded message sizes and deadlines.
- No network, filesystem preopen, WASI capability, or ambient OS handle is granted to the guest by default.
- Platform isolation is capability-gated. If the required OS primitive is unavailable, the host must refuse the task or explicitly mark the run unsupported; it must not silently fall back to in-process execution.
- The fixture runtime remains explicitly non-independent and cannot emit a production-trusted attestation.

## 8. Required negative and adversarial tests

Before any verifier adapter is trusted, tests must demonstrate rejection of:

1. model self-report with no attestation;
2. forged verifier identity or unknown signing key;
3. signature/integrity mismatch;
4. replayed nonce or duplicate attestation;
5. wrong run ID, task ID, predicate digest, rubric or adapter version;
6. expired, future-dated or stale observation;
7. evidence reference substitution or missing evidence;
8. guest attempt to read verifier secrets or mutate verifier storage;
9. verifier timeout, crash, partial response and malformed IPC;
10. unavailable platform isolation or unsupported policy primitive;
11. safety violation combined with an otherwise passing task result;
12. evidence from a prior retry or different model/configuration being reused.

Tests must include real process-boundary and OS-enforcement checks on each supported platform. Unit tests of a trait or in-process mock are necessary but not sufficient.

## 9. Implementation sequence and acceptance gates

1. Freeze task-attempt envelope and canonical serialization/digest rules.
2. Implement host-owned nonce generation, expiry, immutable persistence and replay ledger.
3. Implement one narrowly scoped verifier adapter with an independently observable target.
4. Run verifier outside the guest boundary; protect identity material and IPC.
5. Validate authenticated attestation and three-valued decision semantics.
6. Add adversarial tests, platform enforcement tests and failure-injection tests.
7. Only then connect a model adapter and held-out task runner.
8. Only after matched baselines, trusted evaluator review and sample-size gates may reports contribute to RID publication.

Completion requires documented threat coverage, passing Linux and Windows exact-head CI, platform isolation evidence, replay/failure tests, and independent review of the verifier's actual observation method. A passing mock test or fixture test does not satisfy these gates.

## 10. Explicit non-claims

This contract does not implement signatures, key management, a verifier process, IPC, an OS sandbox, an agent trajectory runner, held-out benchmark execution, trusted evaluator aggregation, or RID scoring. The benchmark manifest must keep `rocksoul.agent-trajectory` planned until all relevant gates are met.
