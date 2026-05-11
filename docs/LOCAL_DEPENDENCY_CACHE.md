# Local Dependency Cache

PrivaGate keeps local development artifacts inside the repository directory to avoid uncontrolled writes to user-level locations.

Ignored local directories:

- `.cargo-home/`
- `.rustup-home/`
- `target/`
- `.cache/`
- `data/`

The scripts `scripts/dev-env.sh` and `scripts/dev-env.ps1` set Cargo and PrivaGate paths for local development.

These directories are local-only and must not be committed.
