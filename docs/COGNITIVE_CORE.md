# Cognitive Core Contracts

This document describes the first implementation layer for Stage 1 of the Rocksoul roadmap.

## Scope

The cognitive core currently provides deterministic, model-independent contracts:

- A turn lifecycle state machine with terminal states and bounded retries.
- A validated turn request and explicit per-turn budgets.
- Structured plan steps and required acceptance checks.
- A structured reflection outcome.
- A cloneable cooperative cancellation signal.
- A turn meter for model-call, output-token, and wall-clock limits.

The REPL now routes each chat request through these contracts as a bounded single-action turn: validate the request and plan, make one model call, then verify that a non-empty assistant response was returned. This is a minimal integration, not multi-step autonomous planning or tool execution.

## Execution contract

A future runtime adapter should:

1. Validate a `TurnRequest` before starting work.
2. Create a `TurnState`, `TurnMeter`, and `CancellationToken` for the turn.
3. Check cancellation and the time/model-call budget before every model request.
4. Record actual model calls and provider-reported output token usage.
5. Parse model-produced plans and reflections into the typed structures, then validate them.
6. Treat model claims as untrusted proposals. Acceptance checks must be evaluated against independently observed runtime evidence.
7. End in exactly one terminal state: completed, failed, or cancelled.

The current REPL integration implements only a subset: one model call per turn and a non-empty-response acceptance check. It does not ask the model for a separate model-generated plan or reflection. Provider-reported completion-token usage is recorded when available.

## Important limitations

- Cancellation is cooperative. An adapter must check the token; it does not forcibly stop an in-flight HTTP request.
- Token accounting depends on actual usage reported by the model provider. The core does not estimate hidden token usage.
- The meter must be called by the adapter. It cannot independently observe network requests. The current REPL records model calls, checks elapsed time after the response, and enforces the output-token limit when the provider returns `usage.completion_tokens`. Providers that omit usage cannot be accounted for and therefore do not receive live output-token enforcement.
- The current CLI has no interactive cancellation command. The cancellation token is available to adapters but is not yet wired to terminal signals or user input.
- Retry policy is currently represented by the turn state's bounded retry counter. Backoff, provider-specific retry classification, and idempotency are not yet implemented.
- No shell, filesystem mutation, tool execution, persistent memory, or self-modification is enabled.

## Validation

The module includes deterministic unit tests for state transitions, invalid plans/reflections, cancellation, and budget enforcement. The repository CI must pass before this increment is considered merged or released.
