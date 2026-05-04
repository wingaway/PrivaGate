# 观测

网关当前内置：

- `tower_http::trace::TraceLayer` HTTP 请求跟踪。
- `tracing_subscriber` JSON 日志。
- `RUST_LOG` / `EnvFilter` 日志级别控制。

OpenTelemetry Collector 样例：

```text
deploy/otel-collector/config.yaml
```

当前服务默认输出结构化 JSON 日志，可由运行环境采集。后续如果启用 OTLP exporter，应保持同一原则：trace、log、metric 只能包含外部可见视图 hash、审计 ID、策略版本和验证结果，不得记录原始数据、本地密钥或 token 映射表。

