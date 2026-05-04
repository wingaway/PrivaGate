# 路线图

本文按当前代码状态整理项目路线。历史研究目标已经保留在白皮书、架构、威胁模型和验证模型中；本文件只记录当前工程推进状态。

## 当前状态

当前项目已直接推进到 Rust 最终产品骨架：

- Rust workspace 已建立。
- `proofgate-core` 已实现隐私核心。
- `proofgate-gateway` 已实现 HTTP 网关。
- 外部模型 adapter 已预留边界，默认不调用任何供应商。
- 构建、测试、Clippy、release build 和主要 HTTP 冒烟测试已通过。

## 已完成

### 文档和研究框架

- 白皮书。
- 架构设计。
- 威胁模型。
- 验证模型。
- 评估计划。
- 技术栈说明。
- 命令手册。
- API 文档。
- 实测计划。
- 完成审计。

### 隐私核心

- 文本、JSON、CSV rows 输入模型。
- 字段级 HMAC tokenization。
- 本地 token 映射日志。
- 外部可见视图生成。
- 输入摘要 hash。
- 外部可见视图 hash。
- 策略版本记录。
- token 可复算。
- 直接标识符残留检查。
- 抑制、透传、时间相对化、地址层级泛化、数值分桶。

### 复杂数据集

- 多表 JSON 数据。
- 跨表 token 一致性。
- 外键完整性验证。
- 关系保持验证。
- 时间顺序验证。
- 中文样例数据。

### 差分隐私统计

- `laplace_count`。
- `laplace_histogram`。
- `laplace_mean`。
- epsilon、delta、beta、sensitivity 记录。
- 隐私预算累计。
- 误差上界计算。

### RAG 与 Agent

- RAG chunk 批量投影。
- chunk 级外部视图 hash。
- 工具输入输出审查。
- 输出残留风险检查。
- 会话级风险累计。

### 工程化

- Rust 隐私核心。
- Rust + axum 网关。
- JSONL 本地映射日志。
- JSONL 审计日志。
- PostgreSQL append-only 审计 sink。
- Docker Compose。
- Kubernetes 部署样例。
- JSON tracing。
- OpenTelemetry Collector 样例。
- 本地缓存和工具链目录固定在项目目录内。

## 当前非目标

- 任意自由文本的无条件完备隐私识别。
- 外部世界知识下的绝对不可重识别。
- 没有任务画像的通用效用保证。
- 依赖外部模型自我声明的安全判断。
- 默认接入任何具体外部模型供应商。

## 后续增强

这些不是当前完成条件，但适合下一轮工程化：

- 策略 JSON Schema 导出。
- 更完整的中文地址解析器。
- 更严格的 typed entity-relation graph schema。
- PostgreSQL 集成测试容器。
- Kubernetes CI schema 校验。
- OTLP exporter 直接集成。
- Python / PyO3 实验绑定。
- 性能基准和压测。
