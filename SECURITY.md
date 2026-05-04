# Security Policy / 安全策略

## English

ProofGate is designed to reduce the information sent from a local trust boundary to an external LLM API. It does not make external providers trustworthy, and it does not turn unsafe data handling into safe data handling by itself.

Please report security issues privately to the repository maintainers once a GitHub repository is created. Do not open public issues for suspected leaks, bypasses, secret exposure, or real-data incidents.

Security-sensitive material that must not be committed:

- Real datasets, prompts, transcripts, model input/output traces, mapping logs, audit logs, or evaluation reports containing real data.
- API keys, bearer tokens, local model credentials, database URLs with passwords, HMAC keys, SSH keys, or cloud credentials.
- Production policies that reveal internal schemas, customer identifiers, account formats, device naming schemes, or operational secrets.

Use `scripts/prepublish-check.*` before release. The scan is heuristic and does not replace manual review.

## 中文

ProofGate 的目标是在本地可信边界内减少发送给外部大模型 API 的信息量。它不假设外部供应商可信，也不能单独把不安全的数据治理流程变成安全流程。

GitHub 仓库创建后，请私下向维护者报告安全问题。疑似泄露、绕过、密钥暴露或真实数据事故不要直接提交公开 issue。

不得提交的安全敏感内容：

- 含真实数据的数据集、prompt、对话记录、模型输入输出 trace、映射日志、审计日志或评估报告。
- API Key、bearer token、本地模型凭据、含密码的数据库 URL、HMAC key、SSH key 或云凭据。
- 会暴露内部 schema、客户标识格式、账户格式、设备命名规则或运行秘密的生产策略。

发布前运行 `scripts/prepublish-check.*`。该扫描是启发式检查，不能替代人工复核。
