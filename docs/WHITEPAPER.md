# PrivaGate Whitepaper

## Abstract

PrivaGate is a local privacy protection and data desensitization gateway for AI and data-processing workflows. Its purpose is to reduce sensitive information exposure when business data flows into less-trusted systems such as external model APIs, agent tools, RAG indexes, analytics services, or cross-boundary application integrations.

The project does not treat hybrid LLM deployment as the goal. Local models, external models, projection layers, reports, audit logs, manual review, and digest binding are implementation mechanisms. The goal is data desensitization, privacy protection, and sensitive information leakage risk control under explicit policies and threat assumptions.

PrivaGate keeps raw data, local mappings, and cryptographic keys inside a local trust boundary. It transforms raw input into an `external_view`, checks whether sensitive values remain, optionally dispatches only that `external_view` to downstream systems, inspects returned outputs, and records evidence for replay and governance.

## Goal

PrivaGate is designed around three primary goals:

1. **Data desensitization.** Replace, generalize, suppress, bucket, or locally retain sensitive values before data leaves the trusted boundary.
2. **Privacy protection.** Reduce the ability of less-trusted observers to recover direct identifiers, link protected values, infer sensitive details, or reconstruct raw records without local keys and mapping tables.
3. **Leakage risk control.** Detect and block sensitive values that appear in projected inputs, model outputs, tool inputs, tool outputs, RAG chunks, or downstream dispatches.

Utility is an operational constraint, not the final objective. PrivaGate should preserve only the minimum fields, relations, events, counts, and labels required for a declared task profile.

## Problem

Given an input dataset `D`, a policy `P`, and a desensitization mechanism `M`, PrivaGate produces an `external_view`:

```text
V = M_P(D)
```

PrivaGate asks three questions:

1. **Desensitization effectiveness.** Were protected values transformed according to policy before crossing the boundary?
2. **Privacy effectiveness.** Under the declared threat model, what can a downstream observer recover, link, or infer from `V` without local secrets?
3. **Minimum utility preservation.** Under the declared task profile, does `V` keep enough structure to perform the task without exposing unnecessary raw data?

These goals are not unconditional. A task that requires exact identity, exact address, exact timestamp, or exact amount may be incompatible with hiding that value from the downstream system. PrivaGate therefore treats each request as a policy-bound privacy and utility trade-off, with explicit residual risks.

## System Model

```text
business input
  -> local detectors or local auxiliary model
  -> PrivaGate policy engine
  -> tokenization / generalization / suppression / bucketing / DP statistics
  -> external view
  -> privacy and utility checks
  -> optional human review gate
  -> optional model, tool, RAG, analytics, or API adapter
  -> output inspection
  -> optional local restoration
  -> audit replay
```

The trusted boundary contains raw input, local keys, mapping tables, policy files, audit storage, local detectors, optional local models, and restoration logic.

Less-trusted or untrusted systems may include external LLM APIs, hosted model gateways, agent tools, RAG indexes, analytics services, external logs, caches, or downstream integrations. These systems should receive only the `external_view` or a narrower derived view.

## Threat Model Summary

The default attacker can observe data sent outside the local boundary and may use public background knowledge for linkage attacks. The attacker cannot access local HMAC keys, raw data, local token mappings, protected audit storage, or local-only model traces.

In scope:

- direct identifier leakage;
- quasi-identifier overexposure;
- deterministic token equality and frequency leakage;
- residual business-context inference;
- model or tool output echoing raw values;
- RAG chunk projection failures;
- policy misconfiguration;
- dispatch before required review;
- dispatch of a different view than the reviewed view.

Out of scope:

- local host compromise;
- local key compromise;
- malicious administrators with mapping-log access;
- workflows that intentionally expose exact sensitive values to a less-trusted system;
- full protection against all public-world linkage attacks.

## Mechanisms

### Typed Tokenization

For direct identifiers such as names, phone numbers, national IDs, account IDs, contract IDs, patient IDs, hostnames, email addresses, claim IDs, and device IDs, PrivaGate uses typed tokenization:

```text
token = HMAC_K(type || canonical(value))
```

If `K` remains local, the token is not reversible by a downstream observer. Deterministic tokenization preserves equality relations but leaks type, equality, and frequency. Random tokenization can reduce frequency leakage but weakens joins and cross-record consistency.

### Generalization and Suppression

For quasi-identifiers and high-risk fields, PrivaGate applies:

- address generalization;
- relative time conversion;
- numeric bucketing;
- rare-attribute suppression;
- local-only handling for fields that should not leave the boundary.

These mechanisms should be selected by policy and justified by the task profile.

### Differential Privacy for Aggregates

For aggregate statistics, PrivaGate can use differential privacy. For a counting query with global sensitivity 1, the Laplace mechanism is:

```text
M(D) = f(D) + Laplace(1 / epsilon)
```

With failure probability `beta`, the absolute error bound is:

```text
abs(M(D) - f(D)) <= ln(1 / beta) / epsilon
```

Reports should include the mechanism, `epsilon`, `delta` when applicable, sensitivity, and error-bound metadata.

### Output and Tool Inspection

Privacy protection does not end at input projection. Downstream systems may echo, reconstruct, or receive sensitive values through tool calls. PrivaGate therefore inspects:

- external model outputs;
- agent tool inputs and outputs;
- RAG chunks before indexing or external use;
- dispatch payloads at adapter boundaries.

Inspection is performed against local mapping logs and configured sensitive-value detectors. Restoration is local and should occur only after inspection and authorization.

### Human Review and Digest Binding

For workflows requiring human approval, PrivaGate binds the review decision to the `external_view`:

```text
approved(audit_id, external_view_digest) = true
```

At dispatch time, the gateway recomputes the digest and permits dispatch only when:

```text
status(audit_id) = approved
and digest(dispatch.external_view) = reviewed_external_view_digest(audit_id)
```

This proves that the dispatched view is the same view that was reviewed. It does not prove that the reviewer made a correct judgment.

### Model Orchestration and Splitting

Model orchestration is a possible implementation technique for leakage risk control, not the purpose of PrivaGate. Local models may improve detection, classify sensitivity, or decide whether a field should remain local. External models may process only `external_view` data.

For high-risk workflows, PrivaGate may split tasks or data across providers to reduce single-observer context completeness. Such splitting should be represented as a route plan with shard digests, provider identities, policies, and residual-risk notes. It should not weaken the core boundary rule: raw data, local keys, and mapping logs remain local.

## Utility Model

PrivaGate preserves utility only as needed for the declared task. For complex data, useful output is not raw text preservation; it is structural fidelity under transformation:

```text
if (u, predicate, v) in R,
then (T(u), predicate, T(v)) in T(R)
```

Task profiles may require:

- entity type preservation;
- equality preservation for selected fields;
- foreign-key validity;
- event-order preservation;
- relation preservation;
- bounded numeric transformation;
- label preservation;
- aggregate accuracy within declared DP bounds.

The task profile should express minimum necessary utility. A stronger privacy posture should be preferred when two projections have equivalent task utility.

This same principle applies to promoted follow-up views produced from local aggregation. A follow-up task profile may require its own local utility gate before the aggregated view is allowed to re-enter a later route plan.

## Evidence and Reports

Reports are not the purpose of PrivaGate. They are the evidence layer that makes privacy protection auditable, replayable, and governable.

Each protected transformation may produce:

- `external_view`: data allowed to cross a specified boundary;
- `privacy_report`: mechanisms, verification results, privacy budget, leakage notes, and residual risks;
- `utility_report`: required field checks, structural constraints, and task-preservation checks;
- `audit_summary`: stable digests, policy version, replay identifiers, and blocked state;
- `manual_review`: optional pending, approved, or rejected review state bound to a digest;
- route or shard evidence when data is split across providers or tools.

Reports are reproducible records of mechanisms, parameters, constraints, checks, and outcomes. They are not external endorsement and do not prove safety outside the configured policy and threat model.

## Evaluation

PrivaGate evaluation should measure privacy protection first and task utility second. Synthetic data should cover structured JSON, free text, multi-table relations, RAG chunks, security logs, Chinese and English identifier formats, direct identifiers, quasi-identifiers, and synthetic secrets.

Privacy metrics:

- raw-value residue in `external_view` objects;
- raw-value residue in downstream outputs;
- token mapping isolation;
- output inspection pass rate;
- tool input and output leakage findings;
- RAG chunk projection failures;
- differential-privacy budget and error bounds;
- known residual leakage such as equality and frequency leakage;
- manual review pass and block behavior.

Utility metrics:

- required field preservation;
- relation preservation;
- foreign-key validity;
- event-order validity;
- task label preservation;
- bounded numeric transformation;
- statistical error bounds for DP outputs.

Evaluation runs should record policy versions, inputs, `external_view` objects, reports, digests, commands, and expected outcomes. Real production data, real credentials, mapping logs, and model traces must not be published.

## Current Scope

The current implementation is a Rust prototype:

- `privagate-core`: tokenization, detectors, projection, verification, reports, and DP helpers.
- `privagate-gateway`: axum HTTP gateway for projection, output inspection, token restoration, statistics, RAG chunk projection, tool inspection, session risk checks, manual review, and a reserved model adapter boundary.
- `config/policy.sample.json`: synthetic policy sample for Chinese and English identifier formats.
- `tests/external_api_simulation`: synthetic-only evaluation for local and external model roles.

The current implementation does not claim absolute privacy for arbitrary free text, does not protect against local key compromise, and does not replace deployment hardening, access control, policy review, or human governance.

## Research Hypothesis

Sensitive data protection for AI and data-processing workflows needs a local privacy gateway rather than a best-effort text filter. Direct identifiers should be handled by cryptographic or deterministic mechanisms when relations must be preserved. Quasi-identifiers should be generalized, suppressed, bucketed, or kept local according to explicit policy. Aggregate statistics should expose privacy budgets and error bounds. Complex business data should be checked through structure-preserving utility constraints.

If every boundary crossing is governed by local desensitization, leakage inspection, and replayable evidence, PrivaGate can reduce sensitive information exposure while preserving the minimum utility needed for downstream tasks.
