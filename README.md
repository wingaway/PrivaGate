# ProofGate

**English.** ProofGate is a verifiable redaction gateway for hybrid LLM deployment. It keeps raw data, local mappings, and cryptographic keys inside a local trust boundary, projects inputs into an external-visible view, sends only that view to an external LLM API, and emits reproducible privacy and utility reports.

**中文。** ProofGate 是面向混合大模型部署的可验证脱敏网关。项目把原始数据、映射表和密钥保留在本地可信边界内，将输入投影为外部可见视图，仅把该视图发送给外部大模型 API，并输出可复算的隐私与效用证明报告。

**Keywords / 关键词：** privacy-preserving LLM, verifiable redaction, data desensitization, hybrid LLM deployment, local model, external LLM API, tokenization, differential privacy, auditability, PII redaction, 隐私保护大模型，可验证脱敏，数据脱敏，混合模型部署，本地模型，外部 API，令牌化，差分隐私，审计复算。

Suggested GitHub topics: `llm`, `privacy`, `redaction`, `tokenization`, `data-privacy`, `privacy-preserving`, `hybrid-llm`, `pii-redaction`, `rust`, `axum`.

## Research Question / 研究问题

ProofGate studies two verifiable questions:

1. **Redaction effectiveness.** Under an explicit threat model, does the external-visible view remove, replace, or generalize protected values so that direct recovery is blocked?
2. **Information utility.** Under an explicit task profile, does the projected view preserve the fields, relations, events, counts, and labels required by the downstream task?

这两个目标不是无条件同时成立的。ProofGate 把每次处理视为一个隐私-效用投影问题，并要求输出可复算的报告，而不是依赖模型自我判断。

## Workflow / 工作流程

```text
raw input
  -> local auxiliary model or local detectors
  -> ProofGate projection
  -> external-visible view + proof reports
  -> external LLM API
  -> output inspection
  -> optional local token restoration
  -> audit replay
```

```text
原始数据
  -> 本地辅助模型或本地检测器
  -> ProofGate 投影
  -> 外部可见视图 + 证明报告
  -> 外部大模型 API
  -> 输出复检
  -> 可选本地 token 复原
  -> 审计复算
```

## What Is Included / 发布内容

- `proofgate-core`: HMAC tokenization, field projection, text detectors, hash binding, privacy reports, utility reports, and verification helpers.
- `proofgate-gateway`: Rust + axum HTTP gateway with `/v1/project`, `/v1/inspect-output`, `/v1/restore-output`, statistics, RAG chunk projection, and reserved model adapter boundary.
- `config/policy.sample.json`: synthetic policy sample for Chinese and English identifiers.
- `examples/`: minimal JSON requests.
- `tests/external_api_simulation/`: synthetic-only evaluation dataset and external API simulation runner.
- `docs/`: whitepaper, architecture, threat model, verification model, API, deployment, evaluation, and release notes.
- `Dockerfile`, `docker-compose.yml`, and `deploy/`: container and Kubernetes examples.

不发布真实数据、密钥、生产策略、模型输入输出 trace、映射日志、审计日志或本地构建缓存。详见 [Open Source Release](docs/OPEN_SOURCE_RELEASE.md)。

## Quick Start / 快速开始

Linux or container-oriented development:

```bash
source ./scripts/dev-env.sh
export PROOFGATE_HMAC_KEY="replace-with-local-test-secret"
./scripts/cargo.sh test
./scripts/cargo.sh run -p proofgate-gateway
```

Windows local development:

```powershell
.\scripts\dev-env.ps1
$env:PROOFGATE_HMAC_KEY="replace-with-local-test-secret"
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 run -p proofgate-gateway
```

Example request:

```bash
curl -sS http://127.0.0.1:8080/v1/project \
  -H 'Content-Type: application/json' \
  --data-binary @examples/project-request.json
```

The response contains:

- `external_view`: the view that may be sent to an external model.
- `privacy_report`: checks for replacement, removal, generalization, and digest binding.
- `utility_report`: checks for required task fields and structural constraints.
- `audit_summary`: replayable identifiers and digests.

响应包含外部可见视图、隐私证明报告、效用证明报告和可复算审计摘要。

## External API Simulation / 外部 API 模拟

The simulation uses two OpenAI-compatible chat completion endpoints:

- a local-model simulation endpoint that receives synthetic raw inputs;
- an external-model simulation endpoint that receives only `external_view`.

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="qwen2.5:7b"

export EXTERNAL_MODEL_BASE_URL="https://external-compatible-provider.example/v1"
export EXTERNAL_MODEL_API_KEY="replace-with-test-key"
export EXTERNAL_MODEL_NAME="replace-with-model"

./scripts/run-external-api-simulation.sh
```

To record full model input/output for a synthetic run:

```bash
./scripts/run-external-api-simulation.sh --record-model-io --model-retries 3
```

The generated `data/` directory is ignored and must not be published.

## Documentation / 文档

| Document | 内容 |
|---|---|
| [Whitepaper / 白皮书](docs/WHITEPAPER.md) | Research claim, mathematical model, system boundary, and scope |
| [Documentation map / 文档导航](docs/DOCUMENTATION.md) | Document groups and maintenance rules |
| [Architecture / 架构](docs/ARCHITECTURE.md) | Components, data flow, trust boundary, and module ownership |
| [Threat model / 威胁模型](docs/THREAT_MODEL.md) | Assets, attackers, attack surfaces, assumptions, and non-goals |
| [Verification model / 验证模型](docs/VERIFICATION_MODEL.md) | Privacy checks, utility checks, structural fidelity, and replay |
| [API](docs/API.md) | Gateway endpoints and request/response examples |
| [Evaluation plan / 评估计划](docs/EVALUATION_PLAN.md) | Privacy, utility, complex dataset, and quality gates |
| [External API simulation / 外部 API 模拟](docs/EXTERNAL_API_SIMULATION_TEST.md) | Synthetic two-model simulation protocol |
| [Open source release / 开源发布](docs/OPEN_SOURCE_RELEASE.md) | Publish list, exclusion list, sensitive-data scan, and GitHub notes |
| [Commands / 命令手册](docs/COMMANDS.md) | Development, build, run, test, and static check commands |
| [Deployment / 部署](docs/DEPLOYMENT.md) | Docker Compose and Kubernetes examples |

## Development Checks / 开发检查

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\prepublish-check.ps1
```

## Scope / 范围

ProofGate is a research prototype and engineering scaffold. It provides reproducible checks under explicit policies and threat assumptions. It does not claim that every downstream inference risk is eliminated, and it does not replace domain review, deployment hardening, or manual release review.

ProofGate 是研究原型和工程骨架。它在明确策略和威胁假设下提供可复算检查，不声称消除所有下游推断风险，也不能替代领域审查、部署加固和人工发布复核。

## Citation and License / 引用与许可证

If you use this project in research, see [CITATION.cff](CITATION.cff).

This project is released under the Apache License 2.0. See [LICENSE](LICENSE).
