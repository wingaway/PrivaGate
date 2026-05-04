# 命令手册

本文记录当前项目推进中确认过的本地命令。除非特别说明，所有命令都在仓库根目录执行：

```powershell
Set-Location E:\CodeHub\ProofGate
```

## 环境初始化

开发前先设置项目内缓存路径，避免依赖、工具链和构建产物写入 C 盘：

```powershell
.\scripts\dev-env.ps1
```

该脚本会设置并创建：

```text
CARGO_HOME=E:\CodeHub\ProofGate\.cargo-home
RUSTUP_HOME=E:\CodeHub\ProofGate\.rustup-home
CARGO_TARGET_DIR=E:\CodeHub\ProofGate\target
PROOFGATE_POLICY_PATH=E:\CodeHub\ProofGate\config\policy.sample.json
PROOFGATE_MAPPING_LOG=E:\CodeHub\ProofGate\data\local-mappings.jsonl
PROOFGATE_AUDIT_LOG=E:\CodeHub\ProofGate\data\audit.jsonl
```

手动设置等价命令：

```powershell
$env:CARGO_HOME="E:\CodeHub\ProofGate\.cargo-home"
$env:RUSTUP_HOME="E:\CodeHub\ProofGate\.rustup-home"
$env:CARGO_TARGET_DIR="E:\CodeHub\ProofGate\target"
$env:PROOFGATE_POLICY_PATH="E:\CodeHub\ProofGate\config\policy.sample.json"
$env:PROOFGATE_MAPPING_LOG="E:\CodeHub\ProofGate\data\local-mappings.jsonl"
$env:PROOFGATE_AUDIT_LOG="E:\CodeHub\ProofGate\data\audit.jsonl"
$env:PROOFGATE_HMAC_KEY="replace-with-local-secret"
```

## Rust 工具链

安装或启用 Rust 工具链前建议先执行：

```powershell
.\scripts\dev-env.ps1
```

然后再安装或启用 Rust 工具链，确保 `RUSTUP_HOME` 和 `CARGO_HOME` 已经指向本项目目录。

安装完成后检查：

```powershell
rustc --version
cargo --version
```

## Cargo Wrapper

优先使用项目 wrapper 执行 Cargo 命令：

```powershell
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 run -p proofgate-gateway
.\scripts\prepublish-check.ps1
```

wrapper 会先加载 `scripts/dev-env.ps1`，再调用 `cargo`。`clippy` 会显式转发到 `cargo-clippy`，避免工具链把参数误路由到 `cargo check`。

等价原生命令：

```powershell
.\scripts\dev-env.ps1
cargo test
cargo-clippy --all-targets -- -D warnings
cargo run -p proofgate-gateway
```

发布前敏感信息和必要文件检查：

```powershell
.\scripts\prepublish-check.ps1
```

## 启动网关

```powershell
.\scripts\dev-env.ps1
$env:PROOFGATE_HMAC_KEY="replace-with-local-secret"
.\scripts\cargo.ps1 run -p proofgate-gateway
```

默认监听：

```text
http://127.0.0.1:8080
```

自定义监听地址：

```powershell
$env:PROOFGATE_BIND="127.0.0.1:8090"
.\scripts\cargo.ps1 run -p proofgate-gateway
```

## API 调用

健康检查：

```powershell
Invoke-RestMethod -Method Get -Uri http://127.0.0.1:8080/healthz
```

单文档投影：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/project `
  -ContentType "application/json" `
  -InFile .\examples\project-request.json
```

多表数据投影：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/project `
  -ContentType "application/json" `
  -InFile .\examples\multitable-project-request.json
```

差分隐私统计：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/statistics `
  -ContentType "application/json" `
  -InFile .\examples\multitable-project-request.json
```

RAG chunk 投影：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/rag/project-chunks `
  -ContentType "application/json; charset=utf-8" `
  -Body '{"chunks":[{"chunk_id":"chunk-1","source_uri":"local://doc-1","content_type":"text","payload":"请联系 13800138000"}]}'
```

Agent 工具输入输出审查：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/tool/inspect `
  -ContentType "application/json" `
  -Body '{"tool_name":"database.query","audit_ids":["<audit-id-from-project-response>"],"input":"tool input text","output":"tool output text"}'
```

会话风险累计：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/session/risk `
  -ContentType "application/json" `
  -Body '{"session_id":"session-1","risk_bound":5.0,"events":[{"external_view_digest":"sha256:example","privacy_budget_epsilon":0.5}]}'
```

外部模型输出检查：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/inspect-output `
  -ContentType "application/json" `
  -Body '{"audit_id":"<audit-id-from-project-response>","output":"model output text"}'
```

本地 token 复原：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/restore-output `
  -ContentType "application/json" `
  -Body '{"audit_id":"<audit-id-from-project-response>","output":"model output with tokens"}'
```

外部模型 adapter 预留接口：

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/model-dispatch `
  -ContentType "application/json" `
  -Body '{"provider":"reserved","task_profile":"contract_risk_review","external_view":{"content_type":"json","payload":{}}}'
```

当前默认 adapter 不调用任何模型供应商，只返回 `dispatched=false`。

两个外部 API 模拟实测：

```powershell
.\scripts\dev-env.ps1

$env:LOCAL_MODEL_BASE_URL="https://local-simulation-provider.example/v1"
$env:LOCAL_MODEL_API_KEY="..."
$env:LOCAL_MODEL_NAME="..."

$env:EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
$env:EXTERNAL_MODEL_API_KEY="..."
$env:EXTERNAL_MODEL_NAME="..."

.\scripts\run-external-api-simulation.ps1

# 需要保存完整模型输入输出时使用：
.\scripts\run-external-api-simulation.ps1 --record-model-io --model-retries 3
```

该测试会把 `tests/external_api_simulation/dataset.json` 中的合成数据发送给两个外部 API。

## Docker

Docker Compose 使用项目目录内 volume：

```powershell
$env:PROOFGATE_HMAC_KEY="replace-with-local-secret"
docker compose up --build
```

停止：

```powershell
docker compose down
```

注意：Docker Desktop 自身的数据根目录需要在 Docker Desktop 设置中迁移到 E 盘或非 C 盘位置；Compose 只能控制本项目 volume 映射。

## 静态检查

在没有 Rust 工具链时，可先做 JSON 和文件检查：

```powershell
Get-Content -Raw config\policy.sample.json | ConvertFrom-Json | Select-Object policy_version,task_profile
Get-Content -Raw examples\project-request.json | ConvertFrom-Json | Select-Object content_type
Get-Content -Raw examples\multitable-project-request.json | ConvertFrom-Json | Select-Object content_type
Select-String -Path crates\**\*.rs -Pattern 'TODO|todo!|unimplemented!|panic!'
```

检查是否已经生成项目内缓存目录：

```powershell
Test-Path .cargo-home
Test-Path .rustup-home
Test-Path target
Test-Path data
```
