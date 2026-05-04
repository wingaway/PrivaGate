# Governance / 治理

## English

ProofGate uses a lightweight maintainer model until the contributor base grows.

### Roles

- **Maintainers** review changes, manage releases, triage issues, and protect the project boundary.
- **Contributors** propose issues, submit pull requests, improve docs, add tests, and share reproducible evaluations.
- **Reviewers** may be invited by maintainers for domain-specific review, especially privacy, security, deployment, or language coverage.

### Decision Rules

Routine changes may be merged after one maintainer approval and passing CI. Routine changes include documentation fixes, synthetic test cases, detector improvements with tests, and small implementation fixes.

The following changes require an RFC or design issue before implementation:

- New redaction mechanisms or privacy claims.
- New external model adapters.
- Policy format changes.
- Audit, mapping, or restoration semantics.
- Benchmark methodology changes.
- Any change that broadens what can be sent to an external model.

Maintainers should prefer small, reversible changes. If reviewers disagree, the decision should be resolved by written evidence: threat model, tests, counterexamples, and reproducible evaluation.

### Merge Criteria

Pull requests should satisfy:

- CI passes.
- No real data, secrets, or generated artifacts are committed.
- Privacy and utility impact is described.
- New behavior has focused tests or an explicit test gap.
- Documentation is updated when public behavior changes.

### Release Criteria

Releases should include:

- A concise change summary.
- Compatibility notes for policies, APIs, and reports.
- Test and prepublish-check status.
- Known limitations and unresolved risks.

## 中文

ProofGate 在贡献者规模扩大前采用轻量维护者模型。

### 角色

- **维护者**：审核变更、管理发布、分拣 issue，并保护项目边界。
- **贡献者**：提出问题、提交 PR、改进文档、增加测试、分享可复现实验。
- **评审者**：维护者可邀请特定领域评审者参与隐私、安全、部署或语言覆盖评审。

### 决策规则

常规变更在 CI 通过且获得一名维护者批准后可以合并。常规变更包括文档修正、合成测试样例、带测试的检测器增强和小型实现修复。

以下变更在实现前需要 RFC 或设计 issue：

- 新脱敏机制或隐私主张。
- 新外部模型 adapter。
- 策略格式变更。
- 审计、映射或复原语义变更。
- benchmark 方法变更。
- 任何会扩大外部模型可见内容的变更。

维护者应优先选择小而可回退的变更。如果评审意见冲突，应通过书面证据解决：威胁模型、测试、反例和可复现实验。

### 合并标准

Pull request 应满足：

- CI 通过。
- 不提交真实数据、密钥或生成物。
- 说明隐私与效用影响。
- 新行为有聚焦测试，或明确说明测试缺口。
- 公共行为变更时同步更新文档。

### 发布标准

发布应包含：

- 简洁的变更摘要。
- 策略、API 和报告的兼容性说明。
- 测试和预发布检查状态。
- 已知限制和未解决风险。
