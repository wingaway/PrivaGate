# ProofGate

ProofGate is a verifiable redaction gateway for hybrid LLM deployment. It keeps raw data, local mappings, and cryptographic keys inside a local trust boundary, projects inputs into an external-visible view, sends only that view to an external LLM API, and emits reproducible privacy and utility reports.

**Keywords:** privacy-preserving LLM, verifiable redaction, data desensitization, hybrid LLM deployment, local model, external LLM API, tokenization, differential privacy, auditability, PII redaction.

Suggested GitHub topics: `llm`, `privacy`, `redaction`, `tokenization`, `data-privacy`, `privacy-preserving`, `hybrid-llm`, `pii-redaction`, `rust`, `axum`.

## Research Question

ProofGate studies two verifiable questions:

1. **Redaction effectiveness.** Under an explicit threat model, does the external-visible view remove, replace, or generalize protected values so that direct recovery is blocked?
2. **Information utility.** Under an explicit task profile, does the projected view preserve the fields, relations, events, counts, and labels required by the downstream task?

These goals are not unconditional. ProofGate treats each request as a privacy-utility projection problem and emits reproducible evidence rather than relying on model self-assessment.

## Workflow

```text
raw input
  -> local auxiliary model or local detectors
  -> ProofGate projection
  -> external-visible view + proof reports
  -> external LLM API
  -> output inspection
  -> optional local token restoration
  -> audit replay
```

## What Is Included

- `proofgate-core`: HMAC tokenization, field projection, text detectors, hash binding, privacy reports, utility reports, and verification helpers.
- `proofgate-gateway`: Rust + axum HTTP gateway with projection, output inspection, restoration, statistics, RAG chunk projection, and a reserved model adapter boundary.
- `config/policy.sample.json`: synthetic policy sample for Chinese and English identifier formats.
- `examples/`: minimal JSON requests.
- `tests/external_api_simulation/`: synthetic-only evaluation dataset and external API simulation runner.
- `docs/`: whitepaper, architecture, threat model, verification model, API, deployment, evaluation, release, and community documentation.
- `Dockerfile`, `docker-compose.yml`, and `deploy/`: container and Kubernetes examples.

Do not publish real data, keys, production policies, model input/output traces, mapping logs, audit logs, or local build caches. See [Open Source Release](docs/OPEN_SOURCE_RELEASE.md).

## Quick Start

Linux or container-oriented development:

```bash
source ./scripts/dev-env.sh
export PROOFGATE_HMAC_KEY="replace-with-local-test-secret"
./scripts/cargo.sh test
./scripts/cargo.sh run -p proofgate-gateway
```

Windows local development:

```powershell
.\scripts\dev-env.ps1
$env:PROOFGATE_HMAC_KEY="replace-with-local-test-secret"
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 run -p proofgate-gateway
```

Example request:

```bash
curl -sS http://127.0.0.1:8080/v1/project \
  -H 'Content-Type: application/json' \
  --data-binary @examples/project-request.json
```

The response contains:

- `external_view`: the view that may be sent to an external model.
- `privacy_report`: checks for replacement, removal, generalization, and digest binding.
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

## Documentation

| Document | Purpose |
|---|---|
| [Whitepaper](docs/WHITEPAPER.md) | Research claim, mathematical model, system boundary, and scope |
| [Documentation Map](docs/DOCUMENTATION.md) | Document groups and maintenance rules |
| [Architecture](docs/ARCHITECTURE.md) | Components, data flow, trust boundary, and module ownership |
| [Threat Model](docs/THREAT_MODEL.md) | Assets, attackers, attack surfaces, assumptions, and non-goals |
| [Verification Model](docs/VERIFICATION_MODEL.md) | Privacy checks, utility checks, structural fidelity, and replay |
| [API](docs/API.md) | Gateway endpoints and request/response examples |
| [Evaluation Plan](docs/EVALUATION_PLAN.md) | Privacy, utility, complex dataset, and quality gates |
| [External API Simulation](docs/EXTERNAL_API_SIMULATION_TEST.md) | Synthetic two-model simulation protocol |
| [Open Source Release](docs/OPEN_SOURCE_RELEASE.md) | Publish list, exclusion list, sensitive-data scan, and GitHub notes |
| [Contributor Tasks](docs/CONTRIBUTOR_TASKS.md) | Good first issues and modular contribution areas |
| [RFC Process](docs/RFC_PROCESS.md) | Design process for privacy, policy, adapter, and benchmark changes |
| [Commands](docs/COMMANDS.md) | Development, build, run, test, and static check commands |
| [Deployment](docs/DEPLOYMENT.md) | Docker Compose and Kubernetes examples |

## Community

ProofGate welcomes small, reviewable contributions based on synthetic data and reproducible checks. Start with [CONTRIBUTING.md](CONTRIBUTING.md), [GOVERNANCE.md](GOVERNANCE.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), and [docs/CONTRIBUTOR_TASKS.md](docs/CONTRIBUTOR_TASKS.md).

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

ProofGate is a research prototype and engineering scaffold. It provides reproducible checks under explicit policies and threat assumptions. It does not claim that every downstream inference risk is eliminated, and it does not replace domain review, deployment hardening, or manual release review.

## Citation and License

If you use this project in research, see [CITATION.cff](CITATION.cff).

This project is released under the Apache License 2.0. See [LICENSE](LICENSE).
