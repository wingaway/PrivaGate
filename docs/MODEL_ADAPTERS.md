# Model Adapters

`/v1/model-dispatch` is a reserved adapter boundary.

Available adapters:

- `disabled`: default adapter. It does not call any model provider and returns `dispatched=false`.
- `dry_run`: local-private adapter for testing. It accepts only `external_view`, returns `dispatched=true`, and emits a structured summary of the projected payload instead of contacting a real provider.

Future adapters may support OpenAI-compatible APIs, local Ollama, local vLLM, or enterprise model gateways.

Every adapter should publish capability metadata so the gateway can decide whether a task is compatible before dispatch. Current metadata fields are:

- `adapter_class`
- `accepts_external_view_only`
- `supports_digest_binding`
- `supports_manual_review_gate`

Task contracts may declare `allowed_adapter_classes`. When present, the gateway blocks `/v1/model-dispatch` unless the adapter metadata matches one of the allowed classes.

The `dry_run` adapter reports `adapter_class=local_private`, so task contracts must allow `local_private` when that adapter is used.

For `/v1/route-plan/execute`, the configured runtime adapter capability must also match each stage's declared `adapter_class`. This prevents a route plan from being validated as one adapter class and executed with another.

The same runtime-capability rule applies to `/v1/shard-plan/execute`. Shard-aware execution also records local aggregation evidence from the adapter outputs that remain inside the local trust boundary, and local aggregation strategies may materialize local-only assembled outputs from those adapter results.

When shard aggregation promotion is enabled, the gateway may also wrap a local aggregation result as a candidate `external_view` for a follow-up task profile. That candidate remains local evidence and is not auto-dispatched through an adapter. In manual review mode it is also not immediately route-ready because it does not yet carry a projection-backed `audit_id`.

`/v1/shard-plan/bind-promotion` is the step that turns a candidate into a new locally bound follow-up view. Before doing so, the gateway rechecks the supplied shard execution evidence and rejects bindings whose dispatch-output digests do not match the supplied outputs. It also applies the promotion utility gate so a follow-up binding is only created when the promoted view still preserves the required fields, and optionally the structural constraints, needed by the next task profile. Task contracts may raise that gate further through `promotion_utility`.

Adapter requirements:

- Accept only `external_view`.
- Accept `audit_id` for dispatch-bound verification.
- Never receive raw input.
- Never receive local mapping logs.
- Never receive local keys.
- Never log API keys or Authorization headers.
- When `PRIVAGATE_REVIEW_MODE=manual`, dispatch only after `/v1/review/approve` has approved the same `audit_id` and `external_view_digest`.
- Include tests proving the adapter boundary.
- Expose stable capability metadata for audit and policy checks.

Changes to adapter semantics require an RFC.

Digest binding prevents a workflow from reviewing one projected view and dispatching a different one. The gateway recomputes the digest of the dispatch request and compares it with the approved review record.
