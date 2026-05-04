# 验证模型

## 目标

验证模型定义如何在数学和工程上检查脱敏网关的可靠性。项目使用两类可复算报告：

- 隐私证明报告。
- 效用证明报告。

证明报告不是人工承诺，而是由机制、参数、输入摘要、外部可见视图摘要和验证结果构成的机器可读事实记录。

## 基本符号

```text
D      原始数据集
M      脱敏机制
V      外部可见视图，V = M(D)
F      下游任务集合
K      本地密钥
T      实体映射函数
G      原始实体关系图
G'     脱敏后的实体关系图
```

## 隐私目标

### 直接标识符

对直接标识符使用 keyed tokenization：

```text
token = HMAC_K(type || canonical(value))
```

验证目标：

```text
external_view does not contain value
token can be recomputed locally from K and value
token cannot be inverted without K under HMAC security assumption
```

需要显式记录泄露面：

- 相等性泄露。
- 频率泄露。
- 类型泄露。
- 长度或格式泄露。

### 随机 token

当任务不需要跨记录一致性时，可使用随机 token：

```text
token = random(type)
```

验证目标：

```text
token uniqueness >= configured bound
token mapping exists only locally
external_view does not reveal equality unless explicitly allowed
```

### 准标识符

准标识符使用泛化、抑制、分桶或差分隐私。

对于统计查询，差分隐私目标为：

```text
Pr[M(D) in S] <= exp(epsilon) * Pr[M(D') in S] + delta
```

验证目标：

```text
mechanism sensitivity is declared
epsilon and delta are recorded
privacy budget composition is tracked
error bound is computable
```

## 效用目标

效用不是对所有可能任务成立，而是对任务集合 `F` 成立：

```text
forall f in F:
  loss(f(V), f(D)) <= alpha_f
```

不同任务使用不同损失函数：

| 任务 | 可验证效用指标 |
|---|---|
| 分类 | 准确率下降、F1 下降 |
| 摘要 | 事实一致性、关键实体角色保持 |
| 抽取 | 字段级 precision、recall、F1 |
| 统计 | 误差上界、置信区间 |
| 合同审查 | 条款关系保持、金额和时间约束 |
| 安全日志 | 拓扑关系保持、事件顺序保持 |
| RAG | 检索召回、答案事实性、引用可追溯 |

## 结构保真

对实体关系图，验证：

```text
forall (u, r, v) in G:
  (T(u), r, T(v)) in G'
```

对多表数据，验证：

```text
foreign_key_valid(G') = true
unique_key_policy_satisfied(G') = true
```

对时间序列，验证：

```text
if t1 < t2 in D:
  T(t1) < T(t2) or relative_order_preserved(T(t1), T(t2))
```

对数值约束，验证：

```text
abs(sum(T(values)) - T(sum(values))) <= configured_bound
```

## 证明报告字段

### 隐私证明报告

```json
{
  "report_type": "privacy_proof",
  "policy_version": "local-policy-version",
  "input_digest": "sha256:...",
  "external_view_digest": "sha256:...",
  "mechanisms": [],
  "privacy_budget": {},
  "declared_leakage": [],
  "residual_risks": [],
  "verification_results": []
}
```

### 效用证明报告

```json
{
  "report_type": "utility_proof",
  "task_profile": "task-name",
  "external_view_digest": "sha256:...",
  "entity_preservation": {},
  "relation_preservation": {},
  "constraint_results": [],
  "statistical_error_bounds": [],
  "task_loss_bounds": []
}
```

## 可复算要求

证明报告必须满足：

- 同一输入、同一策略、同一密钥域下可复算。
- 报告绑定输入摘要和外部可见视图摘要。
- 报告记录策略版本和机制版本。
- 报告记录所有允许泄露面。
- 报告记录未覆盖风险。
- 报告可被自动化测试读取。

## 不可证明边界

以下能力不作为数学保证：

- 本地模型对隐私实体的自由判断。
- 外部模型对结果安全性的自我声明。
- 人工肉眼检查。
- 未形式化的业务经验。
- 未绑定任务集合的泛化效用承诺。

这些能力可以辅助系统运行，但不能作为证明报告的唯一依据。

