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
- JSONL-backed local mapping, audit, and manual-review evidence by default.
- `tokio-postgres` for optional shared PostgreSQL manual-review storage.

## Project Layout

```text
crates/privagate-core/       privacy mechanisms and reports
crates/privagate-gateway/    HTTP gateway
config/                      sample policy
examples/                    projection, route-plan, and shard-plan requests
tests/external_api_simulation/
docs/                        project documentation
deploy/                      deploy/kubernetes and deploy/otel-collector samples
scripts/                     development and simulation wrappers
```

## Operational Characteristics

- The gateway enforces the trust boundary through `external_view`-only dispatch contracts.
- Manual review can be persisted locally through JSONL or shared across instances through PostgreSQL.
- Route-plan and shard-plan execution reuse the same task-contract, review, and adapter-capability checks as direct model dispatch.
- Promotion binding is a local-only workflow that creates a new follow-up `audit_id` after replay and utility verification.

## Runtime Preference

Production-like deployment should use Linux or containers. Windows support is intended for local development.

## Design Preference

Use deterministic, testable mechanisms for privacy boundaries. LLMs may assist with classification, but they should not be the sole privacy boundary.
