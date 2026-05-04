# Observability

ProofGate uses structured tracing through `tracing` and `tower-http`.

Recommended runtime settings:

```bash
export RUST_LOG=info,proofgate_gateway=debug,tower_http=info
```

The sample OpenTelemetry Collector configuration is located at:

```text
deploy/otel-collector/config.yaml
```

Observability must not expose raw input, token mappings, local keys, API keys, or model input/output traces containing real data.
