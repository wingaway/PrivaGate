# 评估计划

## 目标

评估计划用于持续回答两个问题：

1. 脱敏是否有效。
2. 脱敏后的数据是否仍然有效。

评估必须同时覆盖隐私风险、结构保真、统计误差和下游任务表现。

## 数据集类型

### 合成数据

用于早期开发和公开测试：

- 中文姓名。
- 手机号。
- 身份证号。
- 地址。
- 订单号。
- 合同号。
- 金额。
- 时间线。
- 多表外键。
- 工单文本。

合成数据必须保留真实数据形态，但不能包含真实个人信息。

### 半合成复杂数据

使用真实业务 schema 和合成实体值构造：

- 客户表。
- 订单表。
- 合同表。
- 支付表。
- 工单表。
- 日志表。
- 文档片段表。

半合成数据用于验证跨表一致性、关系保真和业务约束。

### 任务基准集

为每类下游任务建立金标：

- 分类标签。
- 抽取字段。
- 摘要关键事实。
- 统计查询答案。
- 图关系查询答案。
- RAG 问答答案。

## 隐私评估

### 直接标识符残留

检查外部可见视图中是否残留原始直接标识符：

```text
residual_direct_identifier_count = 0
```

### token 可复算

验证 token 是否由指定密钥域和规范化函数生成：

```text
recompute(token, K, type, canonical(value)) == true
```

### 准标识符唯一性

计算准标识符组合唯一率：

```text
uniqueness_ratio = unique_qi_records / total_records
```

目标阈值由任务画像决定。

### 链接攻击测试

构造外部攻击数据，尝试将外部可见视图与真实实体链接：

```text
linkage_success_rate <= configured_bound
```

### 多轮拼接测试

检查同一会话多次调用是否累计暴露过多信息：

```text
session_risk_score <= configured_bound
```

### 输出残留测试

检查外部模型输出是否包含未授权敏感内容：

```text
unauthorized_sensitive_output_count = 0
```

## 效用评估

### 实体保持

```text
entity_type_preservation = preserved_entity_types / required_entity_types
```

### 关系保持

```text
relation_preservation = preserved_relations / required_relations
```

### 外键完整

```text
foreign_key_validity = valid_foreign_keys / all_foreign_keys
```

目标通常应为 1.0。

### 时间顺序

```text
time_order_violations = 0
```

### 数值约束

检查金额、数量、合计和区间：

```text
constraint_error <= configured_bound
```

### 统计误差

对差分隐私统计输出，验证误差上界：

```text
absolute_error_bound = ln(1 / beta) / epsilon
```

### 下游任务表现

比较原始视图和脱敏视图上的任务结果：

```text
task_loss <= alpha
```

其中 `alpha` 由任务画像配置。

## 复杂中文场景

必须覆盖中文特有样例：

- 姓名短文本误报。
- 省市区街道地址。
- 身份证号 OCR 错位。
- 手机号空格、横线、全角数字。
- 拼音、谐音和缩写。
- 职务称呼，如张总、王医生、李老师。
- 家庭关系，如我妈、他爱人、孩子。
- 罕见组合，如小城市、特殊职业、精确日期。

## 质量门槛

当前 Rust 网关质量门槛：

| 指标 | 门槛 |
|---|---|
| 直接标识符残留 | 0 |
| 外键完整率 | 1.0 |
| 必要关系保持率 | 1.0 |
| 时间顺序违反 | 0 |
| 证明报告 schema 合法率 | 1.0 |
| token 可复算率 | 1.0 |
| 差分隐私预算记录 | 1.0 |

召回类模型指标不作为数学保证，但应作为辅助质量指标持续跟踪。

## 自动化评估

每次提交应至少运行：

```powershell
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 build --release -p proofgate-gateway
```

接口级回归按 [实测计划](TEST_PLAN.md) 执行，至少覆盖：

- `/v1/project` 单文档投影。
- `/v1/project` 多表投影。
- `/v1/statistics` 差分隐私统计。
- `/v1/inspect-output` 输出残留检查。
- `/v1/restore-output` 本地复原。
- `/v1/rag/project-chunks` RAG chunk 投影。
- `/v1/tool/inspect` Agent 工具输入输出审查。
- `/v1/session/risk` 会话风险累计。
- `/v1/model-dispatch` adapter 禁用边界。

任何外部 API adapter 变更都必须通过输出残留测试和外部可见视图 hash 绑定测试。
