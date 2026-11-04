# Security Policy

Rocksoul is an early-stage agent runtime. The seed release intentionally has a small capability surface.

## Current safeguards

- Model credentials are read from environment variables and are never written by the initializer.
- API keys are sent as bearer credentials only to HTTPS endpoints. Plain HTTP is accepted only for loopback hosts used by local model servers.
- The model client uses bounded request timeouts.
- The seed release has no shell tool, arbitrary code execution, automatic file editing, or self-replacement mechanism.
- Model output is displayed as text. It is not interpreted as a command.

## Current limitations

- Chat history is held in process memory for the current session only.
- The model adapter supports non-streaming OpenAI-compatible chat completions.
- This release does not provide a general-purpose sandbox because it does not execute generated code.
- Environment variables may be visible to other processes or diagnostic tools running under the same user account. Use an appropriate OS-level secret management strategy for sensitive deployments.

## Requirements for future execution and evolution features

Any future capability that runs commands, changes files, installs dependencies, or evolves the core must be designed as a separate security milestone. At minimum it must provide:

1. Explicit capability and permission boundaries.
2. A separate process or container boundary for untrusted code; an in-process language VM is not sufficient by itself.
3. Workspace scoping, resource limits, timeouts, and network restrictions.
4. Immutable snapshots or version-control checkpoints before mutation.
5. Reproducible tests and evaluation before a candidate can be promoted.
6. Human approval for high-impact actions and core replacement.
7. Audit logs, candidate lineage, and rollback.

## Reporting

Please report security issues privately to the repository owner rather than publishing exploit details in an issue.
