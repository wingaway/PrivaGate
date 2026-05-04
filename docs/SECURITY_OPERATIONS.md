# Security Operations

## Secrets

`PROOFGATE_HMAC_KEY` is a local secret. Rotate it according to deployment policy. A key change breaks deterministic token compatibility unless mappings are migrated intentionally.

Never commit:

- HMAC keys;
- API keys;
- bearer tokens;
- database passwords;
- model provider credentials;
- mapping logs;
- audit logs containing real data.

## Mapping Log

`PROOFGATE_MAPPING_LOG` may contain original values and must be encrypted at rest, access-controlled, and retained only as long as needed for inspection and restoration.

## Audit Log

`PROOFGATE_AUDIT_LOG` stores replayable summaries and verification records. `PROOFGATE_AUDIT_POSTGRES_URL` enables the PostgreSQL append-only table `proofgate_audit_log`.

## Output Inspection

External model output should be checked with `/v1/inspect-output` before downstream use. Restoration should happen only inside the local trust boundary through `/v1/restore-output`.

## Operational Rule

If a workflow requires exact sensitive values at the external model, it is outside ProofGate's intended trust model and should be handled locally.
