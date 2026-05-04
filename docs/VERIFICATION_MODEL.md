# Verification Model

## Overview

ProofGate verification is based on reproducible checks over explicit mechanisms and policies. Reports should be machine-readable and tied to stable input and output digests.

## Privacy Verification

Privacy checks may include:

- direct identifier replacement;
- raw-value absence in `external_view`;
- generalized quasi-identifiers;
- suppressed high-risk fields;
- HMAC token domain and field type;
- differential-privacy mechanism and budget;
- output inspection against local mappings.

Each check should emit:

- check name;
- mechanism;
- field or scope;
- pass/fail result;
- evidence summary.

## Utility Verification

Utility checks may include:

- required field presence;
- relation preservation;
- foreign-key validity;
- event-order preservation;
- task label preservation;
- bounded numeric transformation;
- DP error bounds.

Utility is always task-relative. A projection that is useful for contract risk review may be insufficient for customer contact generation.

## Digest Binding

Reports should bind:

```text
input_digest = sha256(canonical_input)
external_view_digest = sha256(canonical_external_view)
```

Digest binding allows later replay and comparison without publishing raw data.

## Manual Review Gate

When `PROOFGATE_REVIEW_MODE=manual`, a human approval is also bound to:

```text
review_binding = (audit_id, external_view_digest, status)
```

The gateway recomputes `external_view_digest` at dispatch time. Dispatch is allowed only when:

```text
status(audit_id) = approved
and digest(dispatch.external_view) = reviewed_external_view_digest(audit_id)
```

This does not prove that a reviewer made a correct judgment; it proves that the external dispatch corresponds exactly to the projected view that was approved.

## Structural Fidelity

For relations:

```text
if (u, predicate, v) exists in raw data,
then (T(u), predicate, T(v)) should exist in projected data
```

This check preserves business structure while hiding original identifiers.

## Differential Privacy

For a counting query with global sensitivity 1:

```text
M(D) = f(D) + Laplace(1 / epsilon)
```

With failure probability `beta`:

```text
abs(M(D) - f(D)) <= ln(1 / beta) / epsilon
```

Reports should include `epsilon`, `delta` when applicable, mechanism name, and error-bound metadata.

## Limitations

Verification is scoped to the configured policy and threat model. It does not prove safety for unspecified tasks, compromised local keys, or unrestricted external linkage attacks.
