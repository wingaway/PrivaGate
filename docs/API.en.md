# API Reference

All examples assume the gateway is running at `http://127.0.0.1:8080`. Requests and responses are JSON unless noted.

## POST /v1/project

Project raw input into an external-visible view and return privacy, utility, and audit reports.

Request:

```json
{
  "content_type": "json",
  "payload": {}
}
```

Supported `content_type` values:

- `json`
- `csv_rows`
- `text`

Response:

```json
{
  "external_view": {},
  "privacy_report": {},
  "utility_report": {},
  "audit_summary": {}
}
```

Local token mappings are never returned in the response. They are written to the local JSONL file configured by `PROOFGATE_MAPPING_LOG`. Audit summaries and verification results are written to `PROOFGATE_AUDIT_LOG`.

## POST /v1/statistics

Run local differential-privacy statistics declared by policy. Row-level data is not sent externally.

Current mechanisms:

- `laplace_count`
- `laplace_histogram`
- `laplace_mean`

Request:

```json
{
  "content_type": "json",
  "payload": {
    "contracts": [],
    "events": [],
    "payments": []
  }
}
```

Response:

```json
{
  "input_digest": "sha256:...",
  "privacy_budget": {
    "epsilon": 2.25,
    "delta": 0.0,
    "consumed": true
  },
  "results": [],
  "verification_results": []
}
```

## POST /v1/inspect-output

Inspect external model output against the local mapping for an `audit_id`.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "output": "model output text"
}
```

Response:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "passed": true,
  "unauthorized_sensitive_output_count": 0,
  "findings": []
}
```

## POST /v1/restore-output

Restore authorized tokens in model output using the local mapping log. This endpoint does not call any external model.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "output": "model output with <PERSON_xxx>"
}
```

Response:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "restored_output": "model output with original value",
  "replacements": 1
}
```

## POST /v1/rag/project-chunks

Project RAG chunks before indexing or before external-context use.

Request:

```json
{
  "chunks": [
    {
      "chunk_id": "chunk-1",
      "source_uri": "local://doc-1",
      "content_type": "text",
      "payload": "please contact +1 650-555-0142"
    }
  ]
}
```

Response:

```json
{
  "chunks": [
    {
      "chunk_id": "chunk-1",
      "source_uri": "local://doc-1",
      "external_view_digest": "sha256:...",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "external_view": {},
      "privacy_passed": true,
      "utility_passed": true
    }
  ]
}
```

## POST /v1/tool/inspect

Inspect agent tool input and output for unauthorized sensitive values associated with previous audit IDs.

Request:

```json
{
  "tool_name": "database.query",
  "audit_ids": ["00000000-0000-0000-0000-000000000000"],
  "input": "tool input text",
  "output": "tool output text"
}
```

## POST /v1/session/risk

Compute session-level exposure and accumulated differential-privacy budget.

Request:

```json
{
  "session_id": "session-1",
  "risk_bound": 5.0,
  "events": [
    {
      "external_view_digest": "sha256:...",
      "privacy_budget_epsilon": 0.5
    }
  ]
}
```

## POST /v1/model-dispatch

Reserved model adapter boundary. The default implementation does not call model providers.

Request:

```json
{
  "provider": "reserved",
  "task_profile": "contract_risk_review",
  "external_view": {
    "content_type": "json",
    "payload": {}
  }
}
```

Response:

```json
{
  "provider": "reserved",
  "dispatched": false,
  "status": "external model adapter is reserved but not configured for task_profile=contract_risk_review",
  "output": null
}
```
