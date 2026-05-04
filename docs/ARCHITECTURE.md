# Architecture

## Goal

ProofGate separates local data protection from external model processing. Raw inputs, token mappings, and keys remain local. External APIs receive only `external_view` plus task instructions.

## Components

- **Gateway API**: HTTP boundary implemented by `proofgate-gateway`.
- **Policy loader**: reads `config/policy.sample.json` or a configured policy path.
- **Projection engine**: implemented by `proofgate-core`; applies tokenization, generalization, suppression, and statistics.
- **Verification engine**: emits privacy and utility reports.
- **Audit writer**: writes append-only local JSONL or PostgreSQL audit records.
- **Mapping writer**: stores token-to-original mappings locally for inspection and restoration.
- **Model adapter boundary**: reserved interface that must receive only `external_view`.

## Data Flow

```text
raw request
  -> content parser
  -> policy selection
  -> projection
  -> privacy and utility verification
  -> external_view
  -> optional external model call
  -> output inspection
  -> optional local restoration
```

## Trust Boundary

Inside the local boundary:

- raw input;
- HMAC key;
- token mapping log;
- audit log;
- policy files;
- optional local auxiliary model.

Outside the local boundary:

- external-visible view;
- external model output;
- digests and non-sensitive report summaries.

## Storage

`PROOFGATE_MAPPING_LOG` points to a local JSONL mapping log. It may contain raw values and must be protected.

`PROOFGATE_AUDIT_LOG` points to a local JSONL audit log. `PROOFGATE_AUDIT_POSTGRES_URL` enables a PostgreSQL append-only audit table named `proofgate_audit_log`.

## Design Constraint

No adapter or tool integration may bypass projection and send raw input, mapping logs, or local keys to an external model.
