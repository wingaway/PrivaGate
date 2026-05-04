# Test Plan

## Local Build Gate

Run:

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/cargo.sh build --release -p proofgate-gateway
```

Windows:

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 build --release -p proofgate-gateway
```

Expected result: formatting passes, tests pass, clippy has no warnings, and release build succeeds.

## API Smoke Test

Start the gateway:

```bash
export PROOFGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p proofgate-gateway
```

Check health:

```bash
curl -sS http://127.0.0.1:8080/healthz
```

Project an example:

```bash
curl -sS http://127.0.0.1:8080/v1/project \
  -H 'Content-Type: application/json' \
  --data-binary @examples/project-request.json
```

Expected result:

- `external_view` contains projected data only.
- `privacy_report.verification_results[*].passed` is true.
- `utility_report.constraint_results[*].passed` is true when constraints are applicable.
- `audit_summary.input_digest` and `audit_summary.external_view_digest` are present.

## Output Inspection

Use the `audit_id` from `/v1/project`:

```bash
curl -sS http://127.0.0.1:8080/v1/inspect-output \
  -H 'Content-Type: application/json' \
  -d '{"audit_id":"<audit-id>","output":"model output text"}'
```

Expected result: `passed=true` when no unauthorized original value appears.

## Restoration

```bash
curl -sS http://127.0.0.1:8080/v1/restore-output \
  -H 'Content-Type: application/json' \
  -d '{"audit_id":"<audit-id>","output":"model output with <PERSON_xxx>"}'
```

Expected result: authorized tokens are restored locally. Restoration must not call an external model.

## Differential Privacy Statistics

Call `/v1/statistics` with synthetic structured data. Expected result:

- privacy budget is recorded;
- DP mechanism name is present;
- verification results are emitted;
- row-level data is not sent externally.

## RAG Chunk Projection

Call `/v1/rag/project-chunks` with synthetic chunks. Expected result:

- each chunk has its own `audit_id`;
- each projected chunk has an `external_view_digest`;
- raw sensitive values are not present in projected chunks.

## Tool Inspection

Call `/v1/tool/inspect` with previous audit IDs and synthetic tool input/output. Expected result: unauthorized sensitive values are reported.

## Session Risk

Call `/v1/session/risk` with synthetic exposure events. Expected result:

- event count is correct;
- epsilon total is correct;
- `passed` reflects the configured risk bound.

## External API Simulation

Run:

```bash
./scripts/run-external-api-simulation.sh --model-retries 3
```

Expected result:

- all synthetic cases pass;
- external raw-value echo count is zero;
- utility score meets the configured threshold;
- reports are written under ignored `data/`.

## Release Gate

Run:

```bash
./scripts/prepublish-check.sh
```

Expected result: no potential secret, real-data artifact, or required-file gap is found.
