# PrivaGate

PrivaGate is a privacy protection and data desensitization gateway for AI and data-processing workflows. It keeps raw data, local mappings, and cryptographic keys inside a local trust boundary, transforms inputs into an `external_view`, sends only that view across less-trusted boundaries, and emits reproducible privacy and utility reports.

**Keywords:** privacy-preserving AI, data desensitization, privacy protection, leakage risk control, tokenization, differential privacy, auditability, local trust boundary, PII redaction, Rust gateway.

Suggested GitHub topics: `privacy`, `data-privacy`, `privacy-preserving`, `pii-redaction`, `tokenization`, `auditability`, `llm`, `ai-security`, `rust`, `axum`.

## Research Question

PrivaGate studies three verifiable questions:

1. **Desensitization effectiveness.** Under an explicit policy, are protected values transformed before data crosses the local trust boundary?
2. **Privacy effectiveness.** Under an explicit threat model, can a downstream observer recover, link, or infer sensitive values without local keys and mapping tables?
3. **Minimum utility preservation.** Under an explicit task profile, does the `external_view` preserve the fields, relations, events, counts, and labels required by the downstream task?

These goals are not unconditional. A task that requires exact sensitive values externally may be incompatible with strong privacy protection. PrivaGate therefore treats each request as a policy-bound privacy and utility trade-off and records machine-readable evidence.

## Workflow

```text
raw input
  -> local auxiliary model or local detectors
  -> PrivaGate projection
  -> optional manual review gate
  -> external view + proof reports
  -> optional model, tool, RAG, analytics, or API dispatch
  -> output inspection
  -> optional local token restoration
  -> audit replay
```

## What Is Included

- `privagate-core`: HMAC tokenization, field projection, text detectors, hash binding, privacy reports, utility reports, and verification helpers.
- `privagate-gateway`: Rust + axum HTTP gateway with projection, output inspection, restoration, statistics, RAG chunk projection, tool inspection, session risk checks, manual review, route planning, shard execution, promotion binding, and a reserved model adapter boundary.
- `config/policy.sample.json`: synthetic policy sample for Chinese and English identifier formats.
- `examples/`: minimal JSON requests for projection, staged routing, shard execution, and promotion binding.
- `tests/external_api_simulation/`: synthetic-only evaluation dataset and external API simulation runner.
- `docs/`: whitepaper, architecture, threat model, verification model, API, deployment, evaluation, release, and community documentation.
- `Dockerfile`, `docker-compose.yml`, and `deploy/`: container and Kubernetes examples.

Do not publish real data, keys, production policies, model input/output traces, mapping logs, audit logs, or local build caches. See [Open Source Release](docs/OPEN_SOURCE_RELEASE.md).

## Quick Start

Linux or container-oriented development:

```bash
source ./scripts/dev-env.sh
export PRIVAGATE_HMAC_KEY="replace-with-local-test-secret"
./scripts/cargo.sh test
./scripts/cargo.sh run -p privagate-gateway
```

Windows local development:

```powershell
.\scripts\dev-env.ps1
$env:PRIVAGATE_HMAC_KEY="replace-with-local-test-secret"
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 run -p privagate-gateway
```

Example request:

```bash
curl -sS http://127.0.0.1:8080/v1/project \
  -H 'Content-Type: application/json' \
  --data-binary @examples/project-request.json
```

The response contains:

- `external_view`: the boundary-crossing view that may leave the local trust boundary.
- `privacy_report`: checks for replacement, removal, generalization, leakage notes, and digest binding.
- `utility_report`: checks for required task fields and structural constraints.
- `audit_summary`: replayable identifiers and digests.

## External API Simulation

The simulation uses two OpenAI-compatible chat completion endpoints:

- a local-model simulation endpoint that receives synthetic raw inputs;
- an external-model simulation endpoint that receives only `external_view`.

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="qwen2.5:7b"

export EXTERNAL_MODEL_BASE_URL="https://external-compatible-provider.example/v1"
export EXTERNAL_MODEL_API_KEY="replace-with-test-key"
export EXTERNAL_MODEL_NAME="replace-with-model"

./scripts/run-external-api-simulation.sh
```

To record full model input/output for a synthetic run:

```bash
./scripts/run-external-api-simulation.sh --record-model-io --model-retries 3
```

Generated data artifacts are ignored by default. Only selected synthetic Markdown reports under `data/complex-text-task-results/*.md` may be published when they carry a public test-data notice. JSON reports, model traces, audit logs, mapping logs, and credentials must stay local.

## Manual Review Mode

Set `PRIVAGATE_REVIEW_MODE=manual` when projected data must be approved by a human before external dispatch. In this mode `/v1/project` returns `manual_review.status="pending"` and `audit_summary.blocked=true`; `/v1/model-dispatch` blocks until `/v1/review/approve` approves the same `audit_id` and `external_view_digest`.

## Documentation

| Document | Purpose |
|---|---|
| [Whitepaper](docs/WHITEPAPER.md) | Goal hierarchy, mathematical model, system boundary, and scope |
| [Documentation Map](docs/DOCUMENTATION.md) | Document groups and maintenance rules |
| [Architecture](docs/ARCHITECTURE.md) | Components, data flow, trust boundary, and module ownership |
| [Threat Model](docs/THREAT_MODEL.md) | Assets, attackers, attack surfaces, assumptions, and non-goals |
| [Verification Model](docs/VERIFICATION_MODEL.md) | Privacy checks, utility checks, structural fidelity, and replay |
| [API](docs/API.md) | Gateway endpoints and request/response examples |
| [Roadmap](docs/ROADMAP.md) | Implemented 2.x slices, next delivery track, and research extensions |
| [Evaluation Plan](docs/EVALUATION_PLAN.md) | Privacy-first evaluation, utility checks, and quality gates |
| [External API Simulation](docs/EXTERNAL_API_SIMULATION_TEST.md) | Synthetic two-model simulation protocol |
| [Open Source Release](docs/OPEN_SOURCE_RELEASE.md) | Publish list, exclusion list, sensitive-data scan, and GitHub notes |
| [Contributor Tasks](docs/CONTRIBUTOR_TASKS.md) | Good first issues and modular contribution areas |
| [RFC Process](docs/RFC_PROCESS.md) | Design process for privacy, policy, adapter, and benchmark changes |
| [Commands](docs/COMMANDS.md) | Development, build, run, test, and static check commands |
| [Deployment](docs/DEPLOYMENT.md) | Docker Compose and Kubernetes examples |

## Community

PrivaGate welcomes small, reviewable contributions based on synthetic data and reproducible checks. Start with [CONTRIBUTING.md](CONTRIBUTING.md), [GOVERNANCE.md](GOVERNANCE.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), and [docs/CONTRIBUTOR_TASKS.md](docs/CONTRIBUTOR_TASKS.md).

Changes that affect privacy claims, utility claims, policy schema, report schema, model adapters, or benchmark methodology should follow the [RFC process](docs/RFC_PROCESS.md).

## Development Checks

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\prepublish-check.ps1
```

## Scope

PrivaGate is a research prototype and engineering scaffold. It provides reproducible checks under explicit policies and threat assumptions. It does not claim that every downstream inference risk is eliminated, and it does not replace domain review, deployment hardening, or manual release review.

## Citation and License

If you use this project in research, see [CITATION.cff](CITATION.cff).

This project is released under the Apache License 2.0. See [LICENSE](LICENSE).
