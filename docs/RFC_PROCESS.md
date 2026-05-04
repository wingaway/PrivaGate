# RFC Process / RFC 流程

## English

RFCs are used for changes that affect ProofGate's privacy boundary, public policy format, report semantics, external model adapters, or benchmark methodology.

Use an RFC when a change:

- Introduces a new redaction, generalization, suppression, tokenization, or differential-privacy mechanism.
- Adds an external model adapter or changes what data an adapter can receive.
- Changes policy schema, report schema, audit semantics, mapping semantics, or restoration semantics.
- Adds a benchmark that will be used to compare project quality.
- Makes a new privacy or utility claim.

### Steps

1. Open an issue using the Research Question template.
2. Copy `docs/rfcs/0000-template.md` into `docs/rfcs/NNNN-title.md`.
3. Fill in motivation, threat model, design, verification method, and compatibility notes.
4. Discuss counterexamples and failure modes before implementation.
5. After maintainer agreement, submit implementation PRs in small parts.

### Required Evidence

An RFC should describe:

- Privacy claim.
- Utility claim.
- Threat assumptions.
- Data that remains local.
- Data that may become externally visible.
- Verification method.
- Tests or benchmark cases.
- Known limitations.

## 中文

RFC 用于影响 ProofGate 隐私边界、公开策略格式、报告语义、外部模型 adapter 或 benchmark 方法的变更。

以下变更需要 RFC：

- 引入新的脱敏、泛化、抑制、tokenization 或差分隐私机制。
- 新增外部模型 adapter，或改变 adapter 可接收的数据范围。
- 改变策略 schema、报告 schema、审计语义、映射语义或复原语义。
- 新增用于比较项目质量的 benchmark。
- 提出新的隐私或效用主张。

### 步骤

1. 使用 Research Question 模板创建 issue。
2. 复制 `docs/rfcs/0000-template.md` 到 `docs/rfcs/NNNN-title.md`。
3. 填写动机、威胁模型、设计、验证方法和兼容性说明。
4. 在实现前讨论反例和失败模式。
5. 维护者同意后，拆分成小 PR 实现。

### 必要证据

RFC 应说明：

- 隐私主张。
- 效用主张。
- 威胁假设。
- 保留在本地的数据。
- 可能外部可见的数据。
- 验证方法。
- 测试或 benchmark 用例。
- 已知限制。
