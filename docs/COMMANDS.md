# Command Reference

Run commands from the repository root.

```powershell
Set-Location <repo-root>
```

Examples below assume the current working directory is your local clone root, regardless of its filesystem path.

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
- `PRIVAGATE_POLICY_PATH`
- `PRIVAGATE_MAPPING_LOG`
- `PRIVAGATE_AUDIT_LOG`
- `PRIVAGATE_REVIEW_LOG`
- `PRIVAGATE_REVIEW_MODE`
- `PRIVAGATE_MODEL_ADAPTER`

## Cargo Wrapper

Windows:

```powershell
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 run -p privagate-gateway
```

Linux:

```bash
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/cargo.sh run -p privagate-gateway
```

## Start the Gateway

```bash
source ./scripts/dev-env.sh
export PRIVAGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p privagate-gateway
```

Default address:

```text
http://127.0.0.1:8080
```

Exercise the dispatch boundary locally without calling a real provider:

```bash
source ./scripts/dev-env.sh
export PRIVAGATE_HMAC_KEY="replace-with-local-secret"
export PRIVAGATE_MODEL_ADAPTER="dry_run"
./scripts/cargo.sh run -p privagate-gateway
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

## Example Route-Plan Validation

```bash
curl -sS http://127.0.0.1:8080/v1/route-plan/validate \
  -H 'Content-Type: application/json' \
  --data-binary @examples/route-plan-request.json
```

PowerShell:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/route-plan/validate `
  -ContentType "application/json" `
  -InFile .\examples\route-plan-request.json
```

Execute a staged dry-run route plan:

```bash
curl -sS http://127.0.0.1:8080/v1/route-plan/execute \
  -H 'Content-Type: application/json' \
  --data-binary @examples/route-plan-execute-request.json
```

PowerShell:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/route-plan/execute `
  -ContentType "application/json" `
  -InFile .\examples\route-plan-execute-request.json
```

Validate a shard-aware plan with local aggregation rules:

```bash
curl -sS http://127.0.0.1:8080/v1/shard-plan/validate \
  -H 'Content-Type: application/json' \
  --data-binary @examples/shard-plan-request.json
```

PowerShell:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/shard-plan/validate `
  -ContentType "application/json" `
  -InFile .\examples\shard-plan-request.json
```

Execute a shard-aware dry-run plan and emit local aggregation evidence:

```bash
curl -sS http://127.0.0.1:8080/v1/shard-plan/execute \
  -H 'Content-Type: application/json' \
  --data-binary @examples/shard-plan-execute-request.json
```

PowerShell:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/shard-plan/execute `
  -ContentType "application/json" `
  -InFile .\examples\shard-plan-execute-request.json
```

Execute a shard-aware dry-run plan that also evaluates a candidate follow-up `external_view`:

```bash
curl -sS http://127.0.0.1:8080/v1/shard-plan/execute \
  -H 'Content-Type: application/json' \
  --data-binary @examples/shard-plan-promote-request.json
```

PowerShell:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/shard-plan/execute `
  -ContentType "application/json" `
  -InFile .\examples\shard-plan-promote-request.json
```

Bind the `claims` promotion candidate from a previous shard execution into a new local follow-up `audit_id`:

```bash
curl -sS http://127.0.0.1:8080/v1/shard-plan/execute \
  -H 'Content-Type: application/json' \
  --data-binary @examples/shard-plan-promote-request.json \
  > target/shard-plan-promote-response.json

jq '{route_plan_execution: .route_plan_execution, aggregation_rules: .aggregation_rules, group_id: "claims"}' \
  target/shard-plan-promote-response.json \
  | curl -sS http://127.0.0.1:8080/v1/shard-plan/bind-promotion \
      -H 'Content-Type: application/json' \
      --data-binary @-
```

PowerShell:

```powershell
$execution = Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/shard-plan/execute `
  -ContentType "application/json" `
  -InFile .\examples\shard-plan-promote-request.json

$bindRequest = @{
  route_plan_execution = $execution.route_plan_execution
  aggregation_rules = $execution.aggregation_rules
  group_id = "claims"
} | ConvertTo-Json -Depth 20

Invoke-RestMethod `
  -Method Post `
  -Uri http://127.0.0.1:8080/v1/shard-plan/bind-promotion `
  -ContentType "application/json" `
  -Body $bindRequest
```

If the bind step returns `binding_created=false`, inspect `promotion.utility_assessment` and `issues` to see whether the promoted follow-up view lost required fields or failed optional structural utility checks.

## Manual Review Mode

Enable the review gate:

```bash
export PRIVAGATE_REVIEW_MODE=manual
export PRIVAGATE_REVIEW_LOG=data/manual-review.jsonl
./scripts/cargo.sh run -p privagate-gateway
```

Approve a projected view after inspecting `external_view`, `privacy_report`, and `utility_report`:

```bash
curl -sS http://127.0.0.1:8080/v1/review/approve \
  -H 'Content-Type: application/json' \
  -d '{"audit_id":"<audit-id>","reviewer":"reviewer-id","reason":"projected view approved"}'
```

PowerShell:

```powershell
$env:PRIVAGATE_REVIEW_MODE="manual"
$env:PRIVAGATE_REVIEW_LOG="data\\manual-review.jsonl"
.\scripts\cargo.ps1 run -p privagate-gateway
```

The manual review store is durable. Reusing the same `PRIVAGATE_REVIEW_LOG` path or PostgreSQL backend keeps pending and approved review decisions across gateway restarts.

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
