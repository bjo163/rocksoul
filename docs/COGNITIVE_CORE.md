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

These are orchestration primitives, not an autonomous agent. The current REPL does not yet use them to run a plan/execute/verify cycle.

## Execution contract

A future runtime adapter should:

1. Validate a `TurnRequest` before starting work.
2. Create a `TurnState`, `TurnMeter`, and `CancellationToken` for the turn.
3. Check cancellation and the time/model-call budget before every model request.
4. Record actual model calls and provider-reported output token usage.
5. Parse model-produced plans and reflections into the typed structures, then validate them.
6. Treat model claims as untrusted proposals. Acceptance checks must be evaluated against independently observed runtime evidence.
7. End in exactly one terminal state: completed, failed, or cancelled.

## Important limitations

- Cancellation is cooperative. An adapter must check the token; it does not forcibly stop an in-flight HTTP request.
- Token accounting depends on actual usage reported by the model provider. The core does not estimate hidden token usage.
- The meter must be called by the adapter. It cannot independently observe network requests.
- Retry policy is currently represented by the turn state's bounded retry counter. Backoff, provider-specific retry classification, and idempotency are not yet implemented.
- No shell, filesystem mutation, tool execution, persistent memory, or self-modification is enabled.

## Validation

The module includes deterministic unit tests for state transitions, invalid plans/reflections, cancellation, and budget enforcement. The repository CI must pass before this increment is considered merged or released.
