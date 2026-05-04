# Tech Stack

## Language and Runtime

- Rust 2021.
- `axum` for HTTP.
- `tokio` for async runtime.
- `serde` and `serde_json` for structured data.
- `tracing` and `tower-http` for observability.

## Core Crates

- `hmac`, `sha2`, and `hex` for HMAC tokenization and digests.
- `regex` for deterministic detectors.
- `uuid` for audit identifiers.
- `chrono` for time handling.
- `tokio-postgres` for optional PostgreSQL audit storage.

## Project Layout

```text
crates/proofgate-core/       privacy mechanisms and reports
crates/proofgate-gateway/    HTTP gateway
config/                      sample policy
examples/                    request examples
tests/external_api_simulation/
docs/                        project documentation
deploy/                      Kubernetes and OpenTelemetry examples
scripts/                     development and simulation wrappers
```

## Runtime Preference

Production-like deployment should use Linux or containers. Windows support is intended for local development.

## Design Preference

Use deterministic, testable mechanisms for privacy boundaries. LLMs may assist with classification, but they should not be the sole privacy boundary.
