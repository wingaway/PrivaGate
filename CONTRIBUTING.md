# Contributing / 贡献

## English

ProofGate is a research-oriented engineering project for verifiable redaction in hybrid LLM deployments. Contributions should keep the system auditable, reproducible, and explicit about privacy-utility trade-offs.

Before opening a pull request:

1. Use synthetic data only. Do not add real personal, customer, patient, employee, account, contract, security, or medical data.
2. Run the local checks:

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

3. Document any new redaction mechanism with its privacy claim, utility claim, threat model, and verification method.
4. Add focused tests for new detectors, projection rules, report fields, or API behavior.
5. Keep generated files out of the repository. `data/`, `target/`, `.cargo-home/`, `.rustup-home/`, `.cache/`, and `__pycache__/` are local artifacts.

## 中文

ProofGate 是面向混合大模型部署的可验证脱敏网关研究工程。贡献应保持系统可审计、可复算，并明确说明隐私与效用之间的权衡。

提交 pull request 前：

1. 仅使用合成数据。不要加入真实个人、客户、患者、员工、账户、合同、安全或医疗数据。
2. 运行本地检查：

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\prepublish-check.ps1
```

3. 新增脱敏机制时，说明隐私主张、效用主张、威胁模型和验证方法。
4. 为新增检测器、投影规则、报告字段或 API 行为添加聚焦测试。
5. 不提交生成物。`data/`、`target/`、`.cargo-home/`、`.rustup-home/`、`.cache/` 和 `__pycache__/` 均为本地文件。
