# Open Source Release

This document defines what should be published to GitHub and what must stay local. The goal is a minimal, reproducible, and searchable research repository.

## Publish

- Source code: `crates/`, `Cargo.toml`, `Cargo.lock`.
- Sample policy: `config/policy.sample.json`.
- Synthetic examples and tests: `examples/`, `tests/external_api_simulation/dataset.json`, `tests/external_api_simulation/run_external_api_simulation.py`.
- Developer scripts: `scripts/*.sh`, `scripts/*.ps1`.
- Deployment examples: `Dockerfile`, `docker-compose.yml`, `deploy/`.
- Documentation: `README.md`, `docs/`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `GOVERNANCE.md`, `CITATION.cff`, `LICENSE`.
- Public synthetic result snapshots: selected `data/complex-text-task-results/*.md` files that carry an explicit synthetic or simulated test-data notice.

## Do Not Publish

- `.env`, `.env.*` except `.env.example`.
- `data/`: audit logs, mapping logs, JSON reports, model input/output traces, and evaluation artifacts, except selected synthetic Markdown result snapshots explicitly listed for publication.
- `target/`, `.cargo-home/`, `.rustup-home/`, `.cache/`, `__pycache__/`, `*.pyc`, logs, and temporary files.
- Real datasets, real prompts, production policies, production schemas, customer identifiers, account formats, hostnames, IP ranges, or internal operating procedures.
- API keys, HMAC keys, bearer tokens, database passwords, SSH keys, cloud credentials, or model-provider credentials.

## Required Checks

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/prepublish-check.sh
```

Windows:

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\prepublish-check.ps1
```

## GitHub Metadata

Recommended repository description:

```text
Privacy protection and data desensitization gateway for AI and data-processing workflows.
```

Recommended topics:

```text
privacy, data-privacy, privacy-preserving, pii-redaction, tokenization,
auditability, llm, ai-security, rust, axum
```
