# Cognitive Core

Rocksoul's cognitive core provides deterministic turn contracts and a bounded model planner/executor.

## Current turn flow

1. Validate the turn request and initialize its state, cancellation token, and budget meter.
2. Make one dedicated model call requesting a JSON plan.
3. Parse and validate the plan, including unique step IDs and required acceptance checks.
4. Reject plans with more than three steps.
5. Execute each step through a separate model call, preserving the ordered conversation context.
6. Reject missing step outputs deterministically.
7. Make a separate reflector model call with the plan, acceptance checks, and execution outputs; validate its structured verdict before completing the turn.

## Budgets

The default turn budget allows at most eight model calls, 16,000 provider-reported output tokens, and 120 seconds elapsed time. The planner, three executor steps, and reflector consume at most five calls. Providers that omit usage metadata cannot be fully token-accounted. The HTTP client also has a 120-second request timeout.

## Safety boundaries

- Plans are untrusted structured proposals, not executable commands.
- The runtime exposes no shell, filesystem mutation, plugins, or arbitrary function tools.
- The reflector reviews the plan's required acceptance checks against the supplied outputs and returns a structured succeeded/needs_retry/failed verdict.
- Reflection is model-based review, not independent evidence: the model can miss errors or accept unsupported claims. No external tool or deterministic domain-specific evaluator verifies factual correctness.
- Cancellation is cooperative but is not yet wired to terminal signals or interactive input.
- Retry backoff and idempotency are not implemented.
- No persistent memory or self-modification is enabled.

## Tests

Unit tests cover structured plan parsing, malformed JSON rejection, and the core lifecycle, validation, cancellation, and budget contracts.
