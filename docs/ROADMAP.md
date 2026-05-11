# Roadmap

## Version 2.x Direction

PrivaGate 2.x is organized around a simple rule: the project goal is data desensitization, privacy protection, and sensitive-information leakage risk control. Projection, reports, audit, review, model adapters, and hybrid routing are supporting mechanisms.

The roadmap is therefore organized around boundary-first capabilities rather than around provider integrations.

## Implemented Foundation

The current 2.x foundation already includes the following delivery slices.

### 2.1 Boundary Hardening

- First-class `local_only` handling for fields that must never cross the trusted boundary.
- Boundary-aware policy evaluation, privacy reports, utility reports, and sample configurations.
- Output inspection, tool inspection, RAG chunk projection, and digest-bound review checks under the same local boundary rule.

### 2.2 Task Contracts

- Task-profile and task-contract checks for minimum necessary utility.
- Required-field and structural utility verification at projection time and before downstream dispatch.
- Clear blocking behavior when a task requires fields intentionally retained as `local_only`.

### 2.3 Audit and Governance

- Persistent local evidence through JSONL logs, with optional PostgreSQL-backed manual review state.
- Replayable audit summaries, digest lineage, route and shard evidence, and promotion-binding evidence.
- Governance-oriented records for review state, blocked dispatches, adapter gates, and residual-risk notes.

### 2.4 Trusted Dispatch and Adapters

- Reserved and dry-run adapter contracts that prove they receive only `external_view`.
- Adapter capability metadata enforced at dispatch time and during route-plan validation.
- Boundary checks shared across model dispatch, staged execution, and shard-aware workflows.

### 2.5 Route Planning, Sharding, and Local Promotion

- `route-plan` validation and staged execution with per-stage digests, adapter-class checks, runtime capability checks, and local evidence capture.
- `shard-plan` validation and execution with shard metadata rules, expected-group checks, local aggregation summaries, aggregation digests, and local-only aggregation outputs.
- Conservative local promotion of aggregated outputs into candidate follow-up `external_view` objects.
- Explicit promotion binding to a new local `audit_id`, including replay verification, dispatch-output digest verification, manual-review creation, and follow-up utility gating.

## Current Capability Summary

- Rust workspace with `privagate-core` and `privagate-gateway`.
- HMAC tokenization for direct identifiers.
- Field projection for structured JSON and text.
- Chinese and English detector coverage for common synthetic identifiers.
- Privacy, utility, and task-contract assessment reports.
- Output inspection and local token restoration.
- Optional manual review gate before model dispatch or follow-up reuse.
- Persistent manual review storage with JSONL or PostgreSQL backends.
- Dry-run adapter for dispatch-boundary validation.
- Route-plan validation and staged execution.
- Shard-plan validation, staged execution, local aggregation, and promotion binding.
- Differential-privacy helpers for selected statistics.
- Synthetic external API simulation with local-model and external-model roles.
- Docker Compose and Kubernetes deployment examples.
- Open-source community templates and RFC process.

## Next Delivery Track

- Harden route and shard workflows from experimental feature status into a stable operator surface, including clearer replay diagnostics and negative-path coverage.
- Extend adapter coverage beyond `dry_run`, while keeping capability enforcement and digest binding identical across adapter classes.
- Package audit, review, and promotion evidence more cleanly for long-running environments and cross-team governance workflows.
- Expand evaluation and benchmark coverage for follow-up utility gates, contextual inference risks, and multilingual structured datasets.
- Tighten policy schema versioning and migration guidance as more task profiles and promotion rules are introduced.

## Research Extensions

- Formalize privacy claims for each mechanism.
- Define attack-case benchmarks for linkage, reconstruction, and contextual inference.
- Improve task utility metrics beyond keyword preservation.
- Explore structured intermediate representations for complex datasets.
- Compare deterministic and random tokenization under different task profiles.
- Evaluate when model splitting or route planning reduces single-observer context completeness without harming minimum required utility.
- Study when local-only retention should force local execution instead of downstream dispatch.
- Define whether task-profile-specific promotion utility profiles should eventually support nested field paths, thresholds, or richer domain-specific checks beyond the current required-field and constraint-check model.

## Non-Goals

- Absolute privacy for arbitrary free text.
- Protection after local key compromise.
- Publishing real datasets or real model traces.
- Letting model adapters bypass `external_view`.
- Replacing deployment hardening, access control, or manual review.
