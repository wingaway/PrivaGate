# Command Reference

Run commands from the repository root.

```powershell
Set-Location E:\CodeHub\ProofGate
```

## Environment

Windows:

```powershell
.\scripts\dev-env.ps1
```

Linux:

```bash
source ./scripts/dev-env.sh
```

The scripts configure:

- `CARGO_HOME`
- `RUSTUP_HOME` when a local toolchain exists
- `CARGO_TARGET_DIR`
- `PROOFGATE_POLICY_PATH`
- `PROOFGATE_MAPPING_LOG`
- `PROOFGATE_AUDIT_LOG`

## Cargo Wrapper

Windows:

```powershell
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 run -p proofgate-gateway
```

Linux:

```bash
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/cargo.sh run -p proofgate-gateway
```

## Start the Gateway

```bash
source ./scripts/dev-env.sh
export PROOFGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p proofgate-gateway
```

Default address:

```text
http://127.0.0.1:8080
```

## Example Projection Request

```bash
curl -sS http://127.0.0.1:8080/v1/project \
  -H 'Content-Type: application/json' \
  --data-binary @examples/project-request.json
```

PowerShell:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/project `
  -ContentType "application/json" `
  -InFile .\examples\project-request.json
```

## External API Simulation

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="qwen2.5:7b"

export EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
export EXTERNAL_MODEL_API_KEY="replace-with-test-key"
export EXTERNAL_MODEL_NAME="replace-with-model"

./scripts/run-external-api-simulation.sh --model-retries 3
```

Use `--record-model-io` only with synthetic data:

```bash
./scripts/run-external-api-simulation.sh --record-model-io --model-retries 3
```

## Docker

```bash
docker compose config
docker compose up --build
docker compose down
```

## Release Checks

```bash
./scripts/prepublish-check.sh
```

Windows:

```powershell
.\scripts\prepublish-check.ps1
```
