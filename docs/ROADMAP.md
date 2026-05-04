# Roadmap

## Current Capabilities

- Rust workspace with `proofgate-core` and `proofgate-gateway`.
- HMAC tokenization for direct identifiers.
- Field projection for structured JSON and text.
- Chinese and English detector coverage for common synthetic identifiers.
- Privacy and utility reports.
- Output inspection and local token restoration.
- Optional manual review gate before model dispatch.
- Differential-privacy helpers for selected statistics.
- Synthetic external API simulation with local-model and external-model roles.
- Docker Compose and Kubernetes examples.
- Open-source community templates and RFC process.

## Near-Term Work

- Improve detector precision and negative examples.
- Add configurable date, location, and numeric generalization policies.
- Add relation-preservation metrics for nested JSON and RAG chunks.
- Add dry-run local model adapters for Ollama and vLLM.
- Persist manual review records beyond single-process memory for clustered deployments.
- Expand multilingual synthetic benchmarks.
- Add release scripts and benchmark summaries.

## Research Work

- Formalize privacy claims for each mechanism.
- Define attack-case benchmarks for linkage and reconstruction.
- Improve task utility metrics beyond keyword preservation.
- Explore structured intermediate representations for complex datasets.
- Compare deterministic and random tokenization under different task profiles.

## Non-Goals

- Absolute privacy for arbitrary free text.
- Protection after local key compromise.
- Publishing real datasets or real model traces.
- Letting model adapters bypass `external_view`.
- Replacing deployment hardening, access control, or manual review.
