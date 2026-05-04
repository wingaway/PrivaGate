# Contributor Tasks

This file lists contribution areas that are useful for ProofGate and small enough to be reviewed incrementally.

## Good First Issues

- Add synthetic examples for new identifier formats.
- Add unit tests for existing text detectors.
- Improve documentation consistency.
- Add comments to complex verification code where they clarify assumptions.
- Add curl and PowerShell examples for existing API endpoints.
- Expand `expected_utility_terms` in synthetic test cases without using real data.

## Detectors

- Enterprise identifiers: tax IDs, invoice numbers, license numbers, and registry numbers.
- English identifiers: tax ID variants, insurance member formats, IBAN-like examples, postal-code patterns.
- Security logs: API key patterns, cloud resource IDs, hostnames, internal domains, IPv6.
- Code repositories: secrets in `.env`, YAML, JSON, TOML, shell, and CI files.

Contribution expectation:

- Add synthetic positive and negative examples.
- Explain false-positive and false-negative risks.
- Add tests and update the policy sample if needed.

## Projection Mechanisms

- Random tokenization for tasks that do not require equality joins.
- Configurable date granularity.
- Location hierarchy generalization.
- Numeric bucketing policies per task profile.
- Suppression policies for high-risk rare attributes.

Contribution expectation:

- Describe the privacy claim and utility trade-off.
- Add verification results to reports.
- Add focused tests and at least one counterexample.

## Utility and Verification

- Foreign-key preservation metrics.
- Event-order validation for nested JSON.
- Relation preservation for RAG chunks.
- Utility scoring for multilingual summaries.
- Output inspection for token echo, raw-value echo, and suspicious reconstruction.

Contribution expectation:

- Define the metric.
- Add deterministic tests.
- Document when the metric is insufficient.

## Model Adapters

- OpenAI-compatible external adapter.
- Local Ollama adapter.
- Local vLLM adapter.
- Dry-run adapter that writes no secrets and returns request summaries.

Contribution expectation:

- Adapter input must be `external_view`, never raw input.
- Do not log API keys or Authorization headers.
- Add tests proving raw input, mappings, and local keys are not sent.

## Benchmarks

- Multilingual synthetic redaction benchmark.
- Complex multi-table relationship benchmark.
- RAG chunk projection benchmark.
- Agent tool input/output inspection benchmark.
- Linkage-attack counterexample benchmark.

Contribution expectation:

- Use synthetic data only.
- Provide expected privacy and utility outputs.
- Record reproducible commands and versioned metrics.

## Documentation

- Keep documentation English-only.
- Add diagrams only when they clarify a data flow or trust boundary.
- Prefer short academic prose over marketing language.
- Keep limitations visible.

## Suggested Labels

- `good first issue`
- `help wanted`
- `detector`
- `policy`
- `verification`
- `benchmark`
- `model-adapter`
- `documentation`
- `security`
- `research`
- `needs-rfc`
