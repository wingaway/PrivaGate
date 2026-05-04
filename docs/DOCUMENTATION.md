# 文档导航

本文按用途整理项目文档，避免研究文档、工程文档和运行文档混在一起。

English readers can start with [WHITEPAPER.en.md](WHITEPAPER.en.md), [API.en.md](API.en.md), and [OPEN_SOURCE_RELEASE.md](OPEN_SOURCE_RELEASE.md). The repository README is bilingual.

## 研究与边界

| 文档 | 用途 |
|---|---|
| [WHITEPAPER.md](WHITEPAPER.md) / [WHITEPAPER.en.md](WHITEPAPER.en.md) | 项目主张、数学保证、系统边界和长期范围 |
| [THREAT_MODEL.md](THREAT_MODEL.md) | 资产、攻击者、攻击面、默认假设和非目标 |
| [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md) | 隐私证明、效用证明、结构保真和可复算报告模型 |
| [EVALUATION_PLAN.md](EVALUATION_PLAN.md) | 隐私、效用、复杂数据和自动化评估计划 |

## 当前工程状态

| 文档 | 用途 |
|---|---|
| [ROADMAP.md](ROADMAP.md) | 当前已完成能力、非目标和后续增强 |
| [TECH_STACK.md](TECH_STACK.md) | 当前 Rust 网关、隐私核心、审计和部署技术栈 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系统组件、数据流、信任边界和模块职责 |
| [API.md](API.md) / [API.en.md](API.en.md) | HTTP API、请求响应和接口语义 |
| [MODEL_ADAPTERS.md](MODEL_ADAPTERS.md) | 外部模型 adapter 边界和默认禁用实现 |
| [EXTERNAL_API_SIMULATION_TEST.md](EXTERNAL_API_SIMULATION_TEST.md) | 两个外部 API 模拟本地模型和外部模型的实测方法 |

## 运行与部署

| 文档 | 用途 |
|---|---|
| [COMMANDS.md](COMMANDS.md) | 常用开发、构建、运行和接口调用命令 |
| [LOCAL_DEPENDENCY_CACHE.md](LOCAL_DEPENDENCY_CACHE.md) | 依赖、工具链、构建产物和缓存位置约束 |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Docker Compose 与 Kubernetes 部署 |
| [SECURITY_OPERATIONS.md](SECURITY_OPERATIONS.md) | 密钥、映射、审计、输出检查和保留策略 |
| [OBSERVABILITY.md](OBSERVABILITY.md) | JSON tracing 与 OpenTelemetry Collector 样例 |

## 验证与交付

| 文档 | 用途 |
|---|---|
| [TEST_PLAN.md](TEST_PLAN.md) | 本地构建、接口、隐私、效用、审计和部署实测流程 |
| [OPEN_SOURCE_RELEASE.md](OPEN_SOURCE_RELEASE.md) | GitHub 发布内容、排除项、敏感信息检查和检索元信息 |

## 更新规则

- 代码接口变更后，同步更新 [API.md](API.md) 和 [TEST_PLAN.md](TEST_PLAN.md)。
- 策略字段或机制变更后，同步更新 [TECH_STACK.md](TECH_STACK.md)、[VERIFICATION_MODEL.md](VERIFICATION_MODEL.md) 和样例策略。
- 部署变量变更后，同步更新 [COMMANDS.md](COMMANDS.md)、[DEPLOYMENT.md](DEPLOYMENT.md)、`.env.example` 和 `docker-compose.yml`。
- 安全边界变更后，同步更新 [THREAT_MODEL.md](THREAT_MODEL.md) 和 [SECURITY_OPERATIONS.md](SECURITY_OPERATIONS.md)。
