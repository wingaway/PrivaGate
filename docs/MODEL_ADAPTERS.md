# Model Adapters

`/v1/model-dispatch` is a reserved adapter boundary. The current default implementation does not call any model provider and returns `dispatched=false`.

Future adapters may support OpenAI-compatible APIs, local Ollama, local vLLM, or enterprise model gateways.

Adapter requirements:

- Accept only `external_view`.
- Never receive raw input.
- Never receive local mapping logs.
- Never receive local keys.
- Never log API keys or Authorization headers.
- Include tests proving the adapter boundary.

Changes to adapter semantics require an RFC.
