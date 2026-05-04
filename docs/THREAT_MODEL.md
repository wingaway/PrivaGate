# Threat Model

## Assets

Protected assets:

- raw input data;
- local HMAC key;
- token mapping log;
- audit log with sensitive summaries;
- production policy files;
- local model prompts and outputs containing raw data;
- credentials for external APIs and storage systems.
- manual review decisions and reviewer metadata.

## Default Attacker

The default attacker can observe data sent to external LLM APIs and may use public background knowledge for linkage attacks. The attacker cannot access local keys, local mapping logs, raw input, or protected local storage.

## Trusted Boundary

Trusted components:

- ProofGate gateway process;
- policy engine;
- local detectors and optional local auxiliary model;
- local key storage;
- local mapping and audit storage.

Untrusted or less-trusted components:

- external LLM API;
- external logs and caches;
- external model outputs;
- downstream tools that have not passed inspection.
- reviewers outside the local trust boundary.

## In-Scope Risks

- Direct identifier leakage.
- Quasi-identifier overexposure.
- Token equality and frequency leakage.
- External model output echoing original values.
- Tool output reintroducing sensitive values.
- RAG chunk projection failures.
- Policy misconfiguration.
- Dispatching a projected view before required human review.
- Dispatching a different projected view than the one that was reviewed.

## Out-of-Scope Risks

- Local host compromise.
- HMAC key compromise.
- Malicious administrators with mapping-log access.
- A task that explicitly requires exposing exact sensitive values externally.
- Full protection against all public-world linkage attacks.

## Required Assumption

ProofGate's privacy claims apply only when raw data, keys, mapping logs, and restoration remain inside the local trust boundary.

Manual review claims apply only when external dispatch goes through ProofGate's model adapter boundary or an equivalent integration that checks the approved `audit_id` and `external_view_digest`.
