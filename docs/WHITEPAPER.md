# ProofGate Whitepaper

## Abstract

External LLM APIs are useful for summarization, reasoning, extraction, and report generation, but enterprise inputs often contain personal data, commercial secrets, medical or financial records, source code, logs, and other restricted information. ProofGate studies a local redaction gateway for hybrid LLM deployment: raw data is processed inside a local trust boundary, projected into an external-visible view, sent to an external model only after projection, and checked again when model output returns.

The goal is not to rely on a model's claim that data is safe. The goal is to produce reproducible privacy and utility reports for every projection.

## Problem

Given an input dataset `D`, a projection mechanism `M`, and an external-visible view `V = M(D)`, ProofGate asks two questions:

1. **Privacy effectiveness.** Under an explicit threat model, can an external observer recover direct identifiers or link protected values without local keys and mapping tables?
2. **Utility preservation.** Under an explicit task profile, does `V` preserve the fields, relations, events, counts, and labels needed by the downstream task?

These objectives are not unconditional. A task that requires exact identity cannot also hide identity from the external model. ProofGate therefore treats each call as a privacy-utility projection problem with machine-readable evidence.

## System Model

```text
business input
  -> local detectors or local auxiliary model
  -> ProofGate policy engine
  -> tokenization / generalization / suppression / DP statistics
  -> external-visible view
  -> external LLM API
  -> output inspection
  -> optional local restoration
  -> audit replay
```

The external model can see prompts, projected context, tool outputs, and model outputs. It cannot access local keys, raw data, or token mapping logs. The local gateway, policy engine, and key material are part of the trusted computing boundary.

## Mechanisms

For direct identifiers such as names, phone numbers, national IDs, account IDs, contract IDs, patient IDs, hostnames, and email addresses, ProofGate uses typed tokenization:

```text
token = HMAC_K(type || canonical(value))
```

If `K` remains local, the token is not reversible by the external API. Deterministic tokenization preserves equality relations but leaks frequency. Random tokenization reduces frequency leakage but weakens cross-record joins.

For quasi-identifiers such as dates, locations, amounts, and rare attributes, ProofGate uses generalization, suppression, bucketing, or local-only handling. For aggregate statistics, ProofGate supports differential privacy. For a counting query with global sensitivity 1, the Laplace mechanism is:

```text
M(D) = f(D) + Laplace(1 / epsilon)
```

With failure probability `beta`, the absolute error bound is:

```text
abs(M(D) - f(D)) <= ln(1 / beta) / epsilon
```

This makes statistical utility auditable.

## Utility Model

For complex data, utility is not raw text preservation. ProofGate uses structural fidelity:

```text
if (u, predicate, v) in R,
then (T(u), predicate, T(v)) in T(R)
```

The projected view should preserve required entity types, foreign-key relations, event order, status transitions, and task labels. Utility is evaluated against explicit task profiles such as contract risk review, healthcare triage, security incident review, RAG projection, and customer support summarization.

## Reports

Each projection produces:

- `external_view`: the projected data that may be sent to the external model.
- `privacy_report`: mechanisms, verification results, privacy budget, leakage notes, and residual risks.
- `utility_report`: required field checks, structural constraints, and task-preservation checks.
- `audit_summary`: stable digests and replay identifiers.

These reports are not external endorsement. They are reproducible records of mechanisms, parameters, constraints, and verification outcomes.

## Current Scope

The current implementation is a Rust prototype:

- `proofgate-core`: tokenization, detectors, projection, verification, reports, and DP helpers.
- `proofgate-gateway`: axum HTTP gateway for projection, output inspection, token restoration, statistics, RAG chunk projection, and reserved model dispatch.
- Synthetic Chinese and English evaluation data for two-model external API simulation.

The project does not claim absolute privacy for arbitrary free text, does not protect against local key compromise, and does not remove the need for deployment hardening or release review.

## Research Hypothesis

Hybrid LLM deployment needs a data projection layer rather than a text-filtering layer. Direct identifiers should be handled by cryptographic or deterministic mechanisms, aggregate statistics should expose explicit privacy budgets, and complex business data should be evaluated by structure-preserving constraints. A reproducible privacy-utility report can become the governance interface between local data systems and external model APIs.
