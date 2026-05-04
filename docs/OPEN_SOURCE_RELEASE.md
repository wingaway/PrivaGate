# Open Source Release / 开源发布

## English

This document defines what should be published to GitHub and what must stay local. The goal is a minimal, reproducible, and searchable research repository.

### Publish

- Source code: `crates/`, `Cargo.toml`, `Cargo.lock`.
- Sample policy: `config/policy.sample.json`.
- Synthetic examples and tests: `examples/`, `tests/external_api_simulation/dataset.json`, `tests/external_api_simulation/run_external_api_simulation.py`.
- Developer scripts: `scripts/*.sh`, `scripts/*.ps1`.
- Deployment examples: `Dockerfile`, `docker-compose.yml`, `deploy/`.
- Documentation: `README.md`, `docs/`, `CONTRIBUTING.md`, `SECURITY.md`, `CITATION.cff`, `LICENSE`.

### Do Not Publish

- `.env`, `.env.*` except `.env.example`.
- `data/`: audit logs, mapping logs, model input/output traces, and evaluation reports.
- `target/`, `.cargo-home/`, `.rustup-home/`, `.cache/`, `__pycache__/`, `*.pyc`, logs, and temporary files.
- Real datasets, real prompts, production policies, production schemas, customer identifiers, account formats, hostnames, IP ranges, or internal operating procedures.
- API keys, HMAC keys, bearer tokens, database passwords, SSH keys, cloud credentials, or model-provider credentials.

### Required Checks

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

Windows:

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\prepublish-check.ps1
```

### GitHub Metadata

Recommended repository description:

```text
Verifiable redaction gateway for hybrid local LLM and external API deployment.
```

Recommended topics:

```text
llm, privacy, redaction, tokenization, data-privacy, privacy-preserving,
hybrid-llm, pii-redaction, rust, axum
```

## 中文

本文定义 GitHub 开源发布时应包含和不应包含的内容。目标是形成一个最小、可复现、易检索的研究仓库。

### 发布

- 源码：`crates/`、`Cargo.toml`、`Cargo.lock`。
- 策略样例：`config/policy.sample.json`。
- 合成样例与测试：`examples/`、`tests/external_api_simulation/dataset.json`、`tests/external_api_simulation/run_external_api_simulation.py`。
- 开发脚本：`scripts/*.sh`、`scripts/*.ps1`。
- 部署样例：`Dockerfile`、`docker-compose.yml`、`deploy/`。
- 文档：`README.md`、`docs/`、`CONTRIBUTING.md`、`SECURITY.md`、`CITATION.cff`、`LICENSE`。

### 不发布

- `.env`、除 `.env.example` 外的 `.env.*`。
- `data/`：审计日志、映射日志、模型输入输出 trace 和评估报告。
- `target/`、`.cargo-home/`、`.rustup-home/`、`.cache/`、`__pycache__/`、`*.pyc`、日志和临时文件。
- 真实数据集、真实 prompt、生产策略、生产 schema、客户标识、账户格式、主机名、IP 段或内部运维流程。
- API Key、HMAC key、bearer token、数据库密码、SSH key、云凭据或模型供应商凭据。

### 必要检查

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

Windows：

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\prepublish-check.ps1
```

### GitHub 元信息

建议仓库描述：

```text
Verifiable redaction gateway for hybrid local LLM and external API deployment.
```

建议 topics：

```text
llm, privacy, redaction, tokenization, data-privacy, privacy-preserving,
hybrid-llm, pii-redaction, rust, axum
```
