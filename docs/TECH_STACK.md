# 技术栈

## 选型原则

技术栈服务于可证明可靠性：

1. 隐私机制可验证。
2. 证明报告可复算。
3. 复杂数据集可结构化处理。
4. 策略和 schema 可版本化。
5. 本地可信边界清晰。
6. 外部模型适配可替换。
7. 依赖缓存和构建产物不得默认落到 C 盘用户目录。

## 当前实现技术栈

| 层 | 当前技术 |
|---|---|
| 网关服务 | Rust + axum |
| 异步运行时 | tokio |
| HTTP 中间件 | tower-http trace |
| 数据模型 | serde / serde_json |
| 加密 tokenization | hmac + sha2 |
| 文本检测 | regex |
| 差分隐私统计 | 本地 Laplace count / histogram / mean |
| 结构约束验证 | 本地确定性验证器 |
| 审计存储 | JSONL + PostgreSQL append-only sink |
| 日志 | tracing + tracing-subscriber JSON |
| 部署 | Docker Compose + Kubernetes YAML |
| 测试 | cargo test + Clippy + HTTP 冒烟 |
| 工具链缓存 | `.cargo-home` / `.rustup-home` / `target` |

## 仓库结构

```text
crates/
  proofgate-core/       # 隐私核心、策略、报告、验证、DP 统计
  proofgate-gateway/    # HTTP API、审计 sink、adapter 边界
config/
  policy.sample.json
examples/
  project-request.json
  multitable-project-request.json
deploy/
  kubernetes/
  otel-collector/
scripts/
  dev-env.sh
  cargo.sh
  run-external-api-simulation.sh
  dev-env.ps1
  cargo.ps1
  run-external-api-simulation.ps1
```

## 外部模型 Adapter

外部模型供应商通过 adapter 边界接入。当前默认实现为 `DisabledModelAdapter`：

- 不读取模型 API key。
- 不发起供应商网络请求。
- 只接受 `external_view`。
- 不接收原始数据、本地密钥或 token 映射表。

## 存储边界

| 存储 | 内容 | 当前实现 |
|---|---|---|
| 映射存储 | token 到原值的本地映射 | `data/local-mappings.jsonl` |
| 审计存储 | hash、策略版本、机制参数、验证结果 | `data/audit.jsonl` 或 PostgreSQL |
| 构建缓存 | Rust 依赖和构建产物 | `.cargo-home`、`.rustup-home`、`target` |

原始数据默认不进入长期业务日志。映射日志包含原始值，必须按安全运行文档限制访问和保留周期。

## 策略格式

当前使用 JSON 策略：

```json
{
  "policy_version": "2026.05.03-production-sample",
  "task_profile": "contract_risk_review",
  "key_domain": "local-kms/proofgate-gateway/hmac/v1",
  "fields": {
    "person_name": {
      "field_type": "person",
      "mechanism": "hmac_token",
      "preserve_equality": true,
      "required_for_task": true
    },
    "event_time": {
      "field_type": "event_time",
      "mechanism": "relative_time",
      "required_for_task": true
    }
  },
  "constraints": {
    "preserve_relations": true,
    "preserve_time_order": true,
    "preserve_foreign_keys": true
  }
}
```

完整样例见 `config/policy.sample.json`。

## 暂不采用

- 纯 LLM 脱敏。
- 只做正则替换。
- 没有中间表示的字符串代理。
- 没有证明报告的 API 网关。
- 没有任务画像的统一脱敏强度。
- 未隔离密钥和映射表的数据库设计。
