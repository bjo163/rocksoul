# Cognitive runtime certification

The runtime is certifiable only when a versioned `CertificationReport` passes every required gate. The report records the source commit, artifact SHA-256, binary identity, operator health, rollback plan, and concrete evidence for each gate.

Required gates are:

- provenance
- policy
- dependencies
- tests
- security audit
- resource bounds
- rollback
- operator health

The verifier fails closed for unsupported schemas, missing identity or rollback evidence, empty gate evidence, and any failed gate. This report is an operational control artifact; it does not promote model output, connector observations, or derived graph data to canonical research truth.
