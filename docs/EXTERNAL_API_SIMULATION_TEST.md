# 外部 API 模拟实测

本文定义使用两个外部 API 模拟“本地辅助模型”和“外部业务模型”的实测方法。由于当前没有真实本地模型，本测试允许把仓库内合成数据发送给两个外部 API。

## 安全边界

- 只能使用 `tests/external_api_simulation/dataset.json` 中的合成数据。
- 不得使用真实个人信息、真实客户号、真实合同号、真实地址或真实业务文本。
- “本地模型模拟 API”会收到原始合成输入，用于模拟本地辅助分类/识别。
- “外部模型模拟 API”只会收到网关生成的 `external_view`。
- 测试报告写入 `data/external-api-simulation-results/`，不提交。

两个 API 需要兼容 OpenAI `POST /chat/completions` 协议。

## 数据集覆盖

当前合成数据集覆盖 15 个用例，其中包含中文、英文和混合结构化场景：

- 单合同结构化脱敏。
- 多客户、多合同、多支付、多事件、多关系边。
- 金融账户与交易风险。
- 医疗分诊摘要。
- 安全日志事件链。
- RAG 文档片段。
- 自由文本手机号、邮箱、身份证检测。
- 客服工单。
- 供应链履约。
- 较大多表统计型数据。
- 英文合同风险，含 full name、SSN、passport、US phone、英文地址。
- 英文医疗理赔，含 member、claim、policy、date of birth、doctor name。
- 英文安全日志，含 user name、employee、host、IPv4、access token。
- 英文自由文本，含 US phone、email、SSN、credit card。
- 英文保险政策，含 national insurance number、UK phone、UK address。

覆盖字段包括姓名、full name、手机号、US/UK phone、邮箱、身份证号、SSN、passport、tax ID、national insurance number、客户号、合同号、订单号、工单号、账户号、银行卡号、credit card、患者号、病历号、claim ID、policy ID、member ID、员工号、设备号、主机名、IP、车牌、公司名、中文地址、英文地址、date of birth、内部 secret、API key、access token、诊断细节、金额桶和相对时间。

## 环境变量

Linux：

```bash
source ./scripts/dev-env.sh

export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="qwen2.5:7b"

export EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
export EXTERNAL_MODEL_API_KEY="..."
export EXTERNAL_MODEL_NAME="..."
```

Windows PowerShell：

```powershell
.\scripts\dev-env.ps1

$env:LOCAL_MODEL_BASE_URL="https://local-simulation-provider.example/v1"
$env:LOCAL_MODEL_API_KEY="..."
$env:LOCAL_MODEL_NAME="..."

$env:EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
$env:EXTERNAL_MODEL_API_KEY="..."
$env:EXTERNAL_MODEL_NAME="..."
```

## 执行

```bash
./scripts/run-external-api-simulation.sh
```

可选参数：

```bash
./scripts/run-external-api-simulation.sh \
  --dataset ./tests/external_api_simulation/dataset.json \
  --gateway-url http://127.0.0.1:8080 \
  --utility-threshold 0.34 \
  --model-retries 3
```

需要记录完整模型输入输出时，显式开启：

```bash
./scripts/run-external-api-simulation.sh --record-model-io --model-retries 3
```

脚本会在网关未启动时自动构建并启动 `proofgate-gateway`。Windows 环境会启动 `proofgate-gateway.exe`。

## 测试流程

每个测试用例执行：

1. 将合成原始输入发送给“本地模型模拟 API”，收集辅助识别结果。
2. 将同一合成输入发送给本地隐私网关 `/v1/project`。
3. 将网关生成的 `external_view` 发送给“外部模型模拟 API”。
4. 调用 `/v1/inspect-output` 检查外部模型输出是否泄露合成原始敏感值。
5. 调用 `/v1/restore-output` 验证 token 复原路径。
6. 计算隐私、效用和网关证明指标。
7. 输出 JSON 和 Markdown 报告。

## 评估方法

| 指标 | 计算方式 | 通过条件 |
|---|---|---|
| 网关隐私证明 | `privacy_report.verification_results[*].passed` | 全部为 true |
| 网关效用约束 | `utility_report.constraint_results[*].passed` | 全部为 true |
| 外部输出残留 | `/v1/inspect-output` | `passed=true` |
| 外部模型原文回显 | 扫描 `expected_sensitive_values` | 计数为 0 |
| 任务效用 | 输出命中 `expected_utility_terms` 比例 | 大于等于 `--utility-threshold` |
| 本地模型原文回显 | 扫描本地模型模拟输出 | 只记录，不作为失败条件 |

本地模型模拟 API 接收的是合成原始输入，因此它回显合成敏感值不判定为系统失败；但该结果会记录在报告中，用于观察“辅助模型”行为。

## 输出

默认输出目录：

```text
data/external-api-simulation-results/
```

文件：

- `external_api_simulation_report.json`
- `external_api_simulation_report.md`
- `model_io_trace.jsonl`：仅在开启 `--record-model-io` 时生成，逐行记录本地模型、脱敏网关和外部模型的输入输出；不记录 API Key 或 Authorization 请求头。

## 失败处理

常见失败：

- 缺少 API 环境变量：设置 `LOCAL_*` 和 `EXTERNAL_*`。
- 外部模型输出泄露合成原始敏感值：检查 prompt、策略和输出检查。
- 网关隐私证明失败：检查策略字段和合成输入是否有未投影引用。
- 效用分数过低：调整任务 prompt 或 `expected_utility_terms`。
