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

## Manual Review

Set `PROOFGATE_REVIEW_MODE=manual` when projected data must be approved by a human before external dispatch.

Operational requirements:

- Review only the projected `external_view`, privacy report, utility report, and digest metadata.
- Do not expose local token mappings, HMAC keys, raw inputs, API keys, or Authorization headers to reviewers unless they are explicitly inside the local trust boundary.
- Approve with `/v1/review/approve` only after the projected view is acceptable for the external model.
- Reject with `/v1/review/reject` when the projected view carries unsafe context or insufficient utility.
- Dispatch through `/v1/model-dispatch` with the same `audit_id`; the gateway recomputes `external_view_digest` and blocks digest mismatches.

## Operational Rule

If a workflow requires exact sensitive values at the external model, it is outside ProofGate's intended trust model and should be handled locally.
