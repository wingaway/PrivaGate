# Model Adapters

`/v1/model-dispatch` is a reserved adapter boundary. The current default implementation does not call any model provider and returns `dispatched=false`.

Future adapters may support OpenAI-compatible APIs, local Ollama, local vLLM, or enterprise model gateways.

Adapter requirements:

- Accept only `external_view`.
- Accept `audit_id` for dispatch-bound verification.
- Never receive raw input.
- Never receive local mapping logs.
- Never receive local keys.
- Never log API keys or Authorization headers.
- When `PROOFGATE_REVIEW_MODE=manual`, dispatch only after `/v1/review/approve` has approved the same `audit_id` and `external_view_digest`.
- Include tests proving the adapter boundary.

Changes to adapter semantics require an RFC.

Digest binding prevents a workflow from reviewing one projected view and dispatching a different one. The gateway recomputes the digest of the dispatch request and compares it with the approved review record.
