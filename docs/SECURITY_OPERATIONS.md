# 安全运行

## 本地可信边界

以下内容必须留在本地可信边界内：

- 原始输入。
- `PROOFGATE_HMAC_KEY`。
- `PROOFGATE_MAPPING_LOG`。
- `PROOFGATE_AUDIT_LOG`。
- 策略文件。

外部模型 adapter 只能接收 `external_view`。

## 日志

当前服务写入两类 JSONL：

- 映射日志：`data/local-mappings.jsonl`
- 审计日志：`data/audit.jsonl`

映射日志包含原始值，必须加密存储、限制访问并设置保留周期。审计日志用于复算、追踪和证明报告归档。

`PROOFGATE_AUDIT_POSTGRES_URL` 用于启用 PostgreSQL append-only 审计后端。未设置时默认写入 JSONL；设置后服务会自动确保 `proofgate_audit_log` 表存在，并把审计记录追加写入 `record jsonb` 字段。

## 密钥轮换

轮换 HMAC key 时必须同步记录：

- 新 `key_domain`。
- 新 `policy_version`。
- 轮换生效时间。
- 旧映射日志的保留和销毁策略。

同一个外部视图 hash 只能绑定生成时使用的策略版本和 key domain。

## 输出检查

所有外部模型输出进入业务系统前必须经过 `/v1/inspect-output`。只有授权复原场景才能调用 `/v1/restore-output`。
