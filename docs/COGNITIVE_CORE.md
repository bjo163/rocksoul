# Cognitive Core Contracts

Rocksoul routes each interactive chat turn through a bounded planner, model-only step executor, and structured reflector. The core remains deterministic and model-independent; model outputs are untrusted proposals.

## Current turn flow

1. Validate the user request and initialize turn state, cancellation token, and budget meter.
2. Ask the configured model for a JSON `TurnPlan`.
3. Parse and validate the plan: non-empty objective, unique step IDs, one to three ordered steps, and at least one required acceptance check.
4. Execute each step through a separate model call, preserving conversation context and step order. No external tools are available.
5. Reject missing or empty step outputs.
6. Ask the model to reflect on the plan, acceptance checks, and generated outputs.
7. Validate the structured reflection. A successful reflection completes the in-memory turn; failed or unresolved reflection fails the turn.

## Budgets

Each turn permits at most five model calls: one planner, up to three step calls, and one reflector. The aggregate output-token ceiling is 16,000 and the wall-clock budget is 120 seconds.

The request's `max_tokens` is conservatively reserved as one fifth of the total budget per call. This ensures the sum of all request caps remains within 16,000 even when a provider omits usage metadata. When usage is returned, the meter also checks the reported aggregate. The HTTP client has a 120-second timeout per request, and the turn meter checks the overall elapsed budget between calls.

## Safety boundaries and limitations

- Plans are untrusted structured proposals, not executable commands.
- The runtime exposes no shell, filesystem mutation, plugins, or arbitrary function tools.
- Step execution is additional model reasoning, not external action execution.
- The reflector is a model reviewing other model outputs. It is not independent evidence and may accept unsupported claims or miss errors.
- No plan acceptance check is currently verified against independent runtime evidence. Therefore planner/reflector success does not award RID XP.
- Cancellation is cooperative but is not yet wired to terminal signals or interactive input.
- Retry backoff, provider-specific retry classification, and automatic re-planning are not implemented.
- Conversation history is session-only. Persistent memory, external tool execution, and self-modification are not enabled.
- All Rocksoul configuration and runtime data paths resolve under the executable root; AppData and user-profile storage are not used.

## Validation

Unit tests cover plan/reflection JSON parsing, fenced JSON, malformed output rejection, unique plan-step validation, lifecycle, cancellation, and budget enforcement. CI must pass formatting, Clippy, tests, offline Core Integrity evaluation, and Windows release build before merge.
