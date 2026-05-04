# API

## POST /v1/project

将原始输入投影为外部可见视图，并返回隐私证明、效用证明和审计摘要。

请求：

```json
{
  "content_type": "json",
  "payload": {}
}
```

`content_type` 支持：

- `json`
- `csv_rows`
- `text`

响应：

```json
{
  "external_view": {},
  "privacy_report": {},
  "utility_report": {},
  "audit_summary": {}
}
```

本地 token 映射不会出现在响应中，只写入 `PROOFGATE_MAPPING_LOG` 指向的本地 JSONL 文件。审计摘要、报告 ID 和验证结果会写入 `PROOFGATE_AUDIT_LOG`。

## POST /v1/statistics

在本地对策略声明的统计项执行差分隐私统计，不外发行级数据。当前支持：

- `laplace_count`
- `laplace_histogram`
- `laplace_mean`

请求：

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

响应：

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

统计结果和预算摘要会写入本地 append-only 审计日志。

## POST /v1/inspect-output

外部模型返回后，按 `audit_id` 读取本地映射，检查输出中是否包含原始敏感值。

请求：

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "output": "model output text"
}
```

响应：

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "passed": true,
  "unauthorized_sensitive_output_count": 0,
  "findings": []
}
```

## POST /v1/rag/project-chunks

对 RAG 入库或检索上下文的 chunk 批量投影，并为每个 chunk 返回外部可见视图 hash 和审计 ID。

请求：

```json
{
  "chunks": [
    {
      "chunk_id": "chunk-1",
      "source_uri": "local://doc-1",
      "content_type": "text",
      "payload": "请联系 13800138000"
    }
  ]
}
```

响应：

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

检查 Agent 工具输入和输出是否包含此前审计 ID 关联的原始敏感值。

请求：

```json
{
  "tool_name": "database.query",
  "audit_ids": ["00000000-0000-0000-0000-000000000000"],
  "input": "tool input text",
  "output": "tool output text"
}
```

响应：

```json
{
  "tool_name": "database.query",
  "passed": true,
  "unauthorized_sensitive_count": 0,
  "findings": []
}
```

## POST /v1/session/risk

计算会话级外部暴露和差分隐私预算累计风险。

请求：

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

响应：

```json
{
  "session_id": "session-1",
  "event_count": 1,
  "exposure_events": 1,
  "epsilon_total": 0.5,
  "risk_score": 1.5,
  "risk_bound": 5.0,
  "passed": true
}
```

## POST /v1/restore-output

按 `audit_id` 从本地映射日志读取 token 映射，将外部模型输出中的授权 token 复原为原始值。

请求：

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "output": "model output with <PERSON_xxx>"
}
```

响应：

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "restored_output": "model output with original value",
  "replacements": 1
}
```

复原只使用本地 `PROOFGATE_MAPPING_LOG`，不会调用外部模型。

## POST /v1/model-dispatch

预留的外部模型 adapter 边界。当前默认实现不调用任何模型供应商。

请求：

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

响应：

```json
{
  "provider": "reserved",
  "dispatched": false,
  "status": "external model adapter is reserved but not configured for task_profile=contract_risk_review",
  "output": null
}
```
