# 实测计划

本文定义项目从本地构建到接口、隐私、效用、审计和部署的实测流程。除非特别说明，命令都在仓库根目录执行：

```powershell
Set-Location E:\CodeHub\ProofGate
```

## 0. 环境前置

目标：确认工具链、依赖缓存和构建产物都在项目目录内。

```powershell
.\scripts\dev-env.ps1
rustc --version
cargo --version
```

验收：

- `CARGO_HOME=E:\CodeHub\ProofGate\.cargo-home`
- `RUSTUP_HOME=E:\CodeHub\ProofGate\.rustup-home`
- `CARGO_TARGET_DIR=E:\CodeHub\ProofGate\target`
- 不使用 C 盘用户目录作为项目依赖缓存。

## 1. 静态质量门

目标：确认代码格式、单元测试、Clippy 和 release 构建通过。

```powershell
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 build --release -p proofgate-gateway
```

验收：

- fmt 无 diff。
- 单元测试全部通过。
- Clippy 无 warning。
- release 二进制构建成功。

## 2. 服务启动

目标：启动本地网关并确认健康检查。

```powershell
.\scripts\cargo.ps1 build -p proofgate-gateway
$env:PROOFGATE_HMAC_KEY="smoke-test-secret"
.\target\debug\proofgate-gateway.exe
```

另开一个 PowerShell：

```powershell
Invoke-RestMethod -Method Get -Uri http://127.0.0.1:8080/healthz
```

验收：

```json
{
  "status": "ok",
  "service": "proofgate-gateway"
}
```

## 3. 单文档投影

目标：验证直接标识符 tokenization、准标识符泛化、证明报告和审计摘要。

```powershell
$single = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/project `
  -ContentType "application/json; charset=utf-8" `
  -InFile .\examples\project-request.json

$single.external_view.payload
$single.privacy_report.verification_results
$single.utility_report
```

验收：

- `person_name`、`phone`、`id_card` 被替换为 typed token。
- `raw_secret` 被抑制。
- `contract_amount_bucket` 输出数值桶，如 `[100000.00,200000.00)`。
- `event_time` 输出相对时间，如 `T0`。
- `address` 输出城市级泛化，如 `上海市`。
- `direct_identifier_residue.passed=true`。
- `input_digest` 和 `external_view_digest` 均以 `sha256:` 开头。

## 4. 多表结构约束

目标：验证跨表 token 一致性、外键完整、关系保持和时间顺序。

```powershell
$multi = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/project `
  -ContentType "application/json; charset=utf-8" `
  -InFile .\examples\multitable-project-request.json

$multi.privacy_report.verification_results
$multi.utility_report.constraint_results
$multi.external_view.payload.events
```

验收：

- `direct_identifier_residue.passed=true`。
- `foreign_key_validity.passed=true`。
- `time_order_validity.passed=true`。
- `relation_preservation.passed=true`。
- 同一 `customer_id` / `contract_id` 在不同表和关系引用中 token 一致。
- 时间输出保持顺序，如 `T0`、`T+95400s`。

## 5. 差分隐私统计

目标：验证本地统计不外发行级数据，并记录预算和误差上界。

```powershell
$stats = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/statistics `
  -ContentType "application/json" `
  -InFile .\examples\multitable-project-request.json

$stats.privacy_budget
$stats.results
$stats.verification_results
```

验收：

- 返回 `laplace_count`、`laplace_histogram`、`laplace_mean` 三类结果。
- `privacy_budget.consumed=true`。
- epsilon 合计等于策略声明总和。
- 每个统计结果包含 `absolute_error_bound`。
- 每个预算验证结果 `passed=true`。

## 6. 输出检查和本地复原

目标：验证外部模型输出返回业务系统前能检查泄露，并可授权复原 token。

```powershell
$auditId = $single.audit_summary.audit_id
$token = $single.external_view.payload.person_name

$restoreBody = @{
  audit_id = $auditId
  output = "hello $token"
} | ConvertTo-Json

$restore = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/restore-output `
  -ContentType "application/json" `
  -Body $restoreBody

$leakBody = '{"audit_id":"' + $auditId + '","output":"leaked 张三"}'

$leak = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/inspect-output `
  -ContentType "application/json; charset=utf-8" `
  -Body $leakBody
```

验收：

- `$restore.replacements=1`。
- 复原结果包含原始值。
- `$leak.passed=false`。
- `$leak.unauthorized_sensitive_output_count=1`。

## 7. RAG 和 Agent 边界

目标：验证 RAG chunk 投影、工具输入输出审查和会话风险累计。

```powershell
$ragBody = '{"chunks":[{"chunk_id":"chunk-1","source_uri":"local://doc-1","content_type":"text","payload":"请联系 13800138000"}]}'

$rag = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/rag/project-chunks `
  -ContentType "application/json; charset=utf-8" `
  -Body $ragBody

$ragAuditId = $rag.chunks[0].audit_id

$toolBody = '{"tool_name":"database.query","audit_ids":["' + $ragAuditId + '"],"input":"","output":"工具返回 13800138000"}'

$tool = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/tool/inspect `
  -ContentType "application/json; charset=utf-8" `
  -Body $toolBody

$riskBody = '{"session_id":"session-1","risk_bound":5.0,"events":[{"external_view_digest":"sha256:example","privacy_budget_epsilon":0.5}]}'

$risk = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/session/risk `
  -ContentType "application/json" `
  -Body $riskBody
```

验收：

- RAG chunk 返回 `external_view_digest`。
- RAG `privacy_passed=true`。
- 工具泄露审查 `$tool.passed=false`。
- `$tool.unauthorized_sensitive_count=1`。
- `$risk.passed=true`。
- `$risk.risk_score=1.5`。

## 8. 外部模型 Adapter 预留

目标：确认模型接口不会误调用供应商。

```powershell
$dispatchBody = @{
  provider = "reserved"
  task_profile = "contract_risk_review"
  external_view = $single.external_view
} | ConvertTo-Json -Depth 20

$dispatch = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/model-dispatch `
  -ContentType "application/json" `
  -Body $dispatchBody
```

验收：

- `$dispatch.dispatched=false`。
- 响应说明 adapter 未配置。
- 不读取任何模型 API key。
- 不发起供应商网络请求。

## 8A. 两个外部 API 模拟本地模型和外部模型

目标：在没有真实本地模型时，用两个外部 OpenAI-compatible API 分别模拟本地辅助模型和外部业务模型。该测试会把仓库内合成数据发送给两个外部 API，不得使用真实数据。

专项文档：

```text
docs/EXTERNAL_API_SIMULATION_TEST.md
```

执行：

```powershell
.\scripts\dev-env.ps1

$env:LOCAL_MODEL_BASE_URL="https://local-simulation-provider.example/v1"
$env:LOCAL_MODEL_API_KEY="..."
$env:LOCAL_MODEL_NAME="..."

$env:EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
$env:EXTERNAL_MODEL_API_KEY="..."
$env:EXTERNAL_MODEL_NAME="..."

.\scripts\run-external-api-simulation.ps1
```

验收：

- `data/external-api-simulation-results/external_api_simulation_report.json` 生成。
- `data/external-api-simulation-results/external_api_simulation_report.md` 生成。
- 网关隐私证明全部通过。
- 外部业务模型输出不包含合成原始敏感值。
- `/v1/inspect-output` 对外部业务模型输出通过。
- 任务效用分数大于等于阈值。

## 9. 审计日志

目标：验证本地映射和 append-only 审计文件写入。

```powershell
Get-Content .\data\local-mappings.jsonl -Tail 5
Get-Content .\data\audit.jsonl -Tail 5
```

验收：

- 映射日志包含 `audit_id`、`field_name`、`field_type`、`token`、`original_value`。
- 审计日志包含 `audit_summary`、报告 ID 和验证结果。
- 映射日志不应外发。

## 10. PostgreSQL 审计后端

目标：验证配置 PostgreSQL URL 后可写入 append-only 审计表。

前置：准备可连接的 PostgreSQL，并设置：

```powershell
$env:PROOFGATE_AUDIT_POSTGRES_URL="host=127.0.0.1 port=5432 user=postgres password=postgres dbname=proofgate"
```

重复执行第 3 节单文档投影。

验收：

```sql
select count(*) from proofgate_audit_log;
select record from proofgate_audit_log order by id desc limit 1;
```

- 表 `proofgate_audit_log` 自动创建。
- 每次处理追加一行 JSONB 记录。
- 不更新或删除旧记录。

## 11. 部署配置

目标：验证 Docker Compose 和 Kubernetes 样例至少能通过本地配置检查。

```powershell
$env:PROOFGATE_HMAC_KEY="compose-static-check"
docker compose config
```

验收：

- Compose 输出包含 `/app/config`、`/app/data`、`/workspace/.cargo-home`、`/workspace/target` 映射。
- `PROOFGATE_HMAC_KEY` 来自环境变量。

Kubernetes：

```powershell
kubectl apply --dry-run=client --validate=false -f .\deploy\kubernetes\proofgate-gateway.yaml
```

说明：无可用 Kubernetes API server 时，`kubectl` 仍可能尝试 discovery 并失败。生产环境应在实际集群上执行 server-side dry-run 或 CI schema 校验。

## 12. 回归汇总命令

快速回归：

```powershell
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 build --release -p proofgate-gateway
```

主端点冒烟摘要：

```powershell
$health = Invoke-RestMethod -Method Get -Uri http://127.0.0.1:8080/healthz
$single = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:8080/v1/project -ContentType "application/json; charset=utf-8" -InFile .\examples\project-request.json
$multi = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:8080/v1/project -ContentType "application/json; charset=utf-8" -InFile .\examples\multitable-project-request.json
$stats = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:8080/v1/statistics -ContentType "application/json" -InFile .\examples\multitable-project-request.json

[pscustomobject]@{
  health = $health.status
  singlePrivacy = $single.privacy_report.verification_results[0].passed
  multiPrivacy = $multi.privacy_report.verification_results[0].passed
  multiConstraintsFailed = ($multi.utility_report.constraint_results | Where-Object { -not $_.passed } | Measure-Object).Count
  statsCount = $stats.results.Count
  statsBudget = $stats.privacy_budget.epsilon
} | ConvertTo-Json -Depth 20
```
