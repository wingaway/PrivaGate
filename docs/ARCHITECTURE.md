# Architecture

## Goal

PrivaGate protects sensitive data before it crosses from a local trusted boundary into less-trusted systems. Raw inputs, token mappings, and keys remain local. Downstream systems receive only `external_view` or a narrower derived view plus task instructions allowed by policy.

## Components

- **Gateway API**: HTTP boundary implemented by `privagate-gateway`.
- **Policy loader**: reads `config/policy.sample.json` or a configured policy path.
- **Projection engine**: implemented by `privagate-core`; applies tokenization, local-only retention, generalization, suppression, bucketing, and differential-privacy statistics.
- **Verification engine**: emits privacy and utility reports tied to stable digests.
- **Manual review gate**: optional human approval boundary for projected views before dispatch.
- **Audit writer**: writes append-only local JSONL or PostgreSQL audit records.
- **Mapping writer**: stores token-to-original mappings locally for inspection and restoration.
- **Dispatch adapter boundary**: reserved interface for model, tool, analytics, or API dispatch that must receive only `external_view`, expose capability metadata, and honor task-contract compatibility checks.
- **Route-plan validator and executor**: validates multi-stage or multi-provider plans against task contracts, adapter classes, manual review readiness, and digest bindings, then can run a bounded staged execution path through the configured adapter while keeping local evidence.
- **Shard-plan summarizer**: adds shard-group semantics, local aggregation rules, aggregation digests, and optional promotion-readiness checks so split workflows can be checked and evidenced before any real multi-provider orchestration is introduced.

## Data Flow

```text
raw request
  -> content parser
  -> policy selection
  -> projection
  -> privacy and utility verification
  -> external_view
  -> optional manual review gate
  -> optional route-plan validation
  -> optional route-plan execution
  -> optional shard-plan validation
  -> optional shard-plan execution and local aggregation evidence
  -> optional promotion binding for a follow-up local view
  -> optional downstream dispatch
  -> output inspection
  -> optional local restoration
```

When `PRIVAGATE_REVIEW_MODE=manual`, projection creates a pending review record bound to `audit_id` and `external_view_digest`. The dispatch adapter boundary blocks dispatch unless the review record is approved and the external view digest exactly matches the approved digest.

When a workflow is split into multiple stages, route-plan validation computes a digest for every stage view, checks whether the declared adapter class is allowed for that task profile, and records local evidence before any downstream stage is attempted.

Route-plan execution replays those checks, compares the declared stage adapter class with the configured runtime adapter capability, and can halt later stages if an earlier stage is blocked or not dispatched.

Shard-plan validation and execution build on the route-plan path by requiring shard metadata when configured, checking expected shard groups, checking provider diversity when required, and computing local aggregation digests from stage bindings and stage output digests.

When shard execution is complete enough for local assembly, the gateway can also materialize a local-only aggregated output according to the configured aggregation strategy. Current strategies include digest-only evidence, collected outputs, object assembly keyed by shard ID, and text concatenation. These outputs remain inside the local trust boundary.

If explicitly configured, the gateway can also assess whether a local aggregation output is eligible to become a candidate `external_view` for a later stage. This promotion assessment remains local, is bounded by task-contract, task-profile-specific follow-up utility checks, and size constraints, and does not auto-dispatch the result.

When a caller decides to continue from that candidate, `/v1/shard-plan/bind-promotion` creates a new local audit binding for the aggregated view. Before binding it, the gateway replays stage-output digests and rechecks the promoted view against the follow-up utility gate. In manual-review mode, a successful binding also creates a new pending review record for the promoted follow-up view.

## Trust Boundary

Inside the local boundary:

- raw input;
- HMAC key;
- token mapping log;
- audit log;
- manual review records;
- policy files;
- local detectors;
- optional local auxiliary model;
- restoration logic.

Outside the local boundary:

- `external_view`;
- downstream model, tool, analytics, or API outputs;
- digests and non-sensitive report summaries.

## Storage

`PRIVAGATE_MAPPING_LOG` points to a local JSONL mapping log. It may contain raw values and must be protected.

`PRIVAGATE_AUDIT_LOG` points to a local JSONL audit log. `PRIVAGATE_AUDIT_POSTGRES_URL` enables a PostgreSQL append-only audit table named `privagate_audit_log`.

`PRIVAGATE_REVIEW_LOG` points to a local JSONL review store. `PRIVAGATE_REVIEW_POSTGRES_URL` enables a PostgreSQL-backed manual review store. Route-plan evidence is written to the same audit backend as other audit events.

## Design Constraint

No adapter or tool integration may bypass projection and send raw input, mapping logs, or local keys to a less-trusted system.

In manual review mode, no adapter may send even a projected view unless the human review gate has approved the exact digest that is being dispatched.
