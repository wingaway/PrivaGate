# Contributing

PrivaGate is a research-oriented engineering project for privacy protection and data desensitization in AI and data-processing workflows. Contributions should keep the system auditable, reproducible, and explicit about privacy-utility trade-offs.

Before opening a pull request:

1. Use synthetic data only. Do not add real personal, customer, patient, employee, account, contract, security, medical, or financial data.
2. Run the local checks:

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

3. Document any new redaction mechanism with its privacy claim, utility claim, threat model, and verification method.
4. Add focused tests for new detectors, projection rules, report fields, or API behavior.
5. Keep generated files out of the repository. `data/`, `target/`, `.cargo-home/`, `.rustup-home/`, `.cache/`, and `__pycache__/` are local artifacts.

Good starting points are listed in [docs/CONTRIBUTOR_TASKS.md](docs/CONTRIBUTOR_TASKS.md).

Open an RFC before changing privacy claims, utility claims, policy schema, report schema, external model adapters, audit semantics, mapping semantics, restoration semantics, or benchmark methodology. See [docs/RFC_PROCESS.md](docs/RFC_PROCESS.md).
