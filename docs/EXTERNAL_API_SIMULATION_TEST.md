# External API Simulation Test

This test uses two OpenAI-compatible APIs to simulate a local auxiliary model and an external business model. Because no real local model is required, the local-model simulation endpoint may receive the repository's synthetic raw inputs.

## Safety Boundary

- Use only `tests/external_api_simulation/dataset.json`.
- Do not use real personal, customer, patient, employee, account, contract, address, operational, or business data.
- The local-model simulation API receives synthetic raw input.
- The external-model simulation API receives only `external_view`.
- Reports are written to `data/external-api-simulation-results/`, which is ignored and must not be committed.

Both APIs must support OpenAI-compatible `POST /chat/completions`.

## Dataset Coverage

The synthetic dataset covers 15 cases across Chinese, English, and mixed structured scenarios:

- single contract projection;
- multi-customer and multi-contract data;
- finance account and transaction risk;
- healthcare triage;
- security event chains;
- RAG document chunks;
- freeform text;
- customer support tickets;
- supply-chain fulfillment;
- larger multi-table statistical data;
- English contract risk;
- English healthcare claims;
- English security logs;
- English support text;
- English insurance policy review.

Covered fields include names, full names, phones, emails, national IDs, SSNs, passports, customer IDs, contract IDs, account IDs, card numbers, patient IDs, claim IDs, policy IDs, member IDs, device IDs, hostnames, IP addresses, addresses, dates of birth, synthetic secrets, API-key-like values, access-token-like values, diagnosis details, amount buckets, and event times.

## Environment

Linux:

```bash
source ./scripts/dev-env.sh

export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="qwen2.5:7b"

export EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
export EXTERNAL_MODEL_API_KEY="replace-with-test-key"
export EXTERNAL_MODEL_NAME="replace-with-model"
```

Windows PowerShell:

```powershell
.\scripts\dev-env.ps1

$env:LOCAL_MODEL_BASE_URL="https://local-simulation-provider.example/v1"
$env:LOCAL_MODEL_API_KEY="replace-with-test-key"
$env:LOCAL_MODEL_NAME="replace-with-model"

$env:EXTERNAL_MODEL_BASE_URL="https://external-provider.example/v1"
$env:EXTERNAL_MODEL_API_KEY="replace-with-test-key"
$env:EXTERNAL_MODEL_NAME="replace-with-model"
```

## Run

```bash
./scripts/run-external-api-simulation.sh --model-retries 3
```

Optional full synthetic model I/O trace:

```bash
./scripts/run-external-api-simulation.sh --record-model-io --model-retries 3
```

The script builds and starts `privagate-gateway` automatically when the gateway is not already healthy.

When `PRIVAGATE_REVIEW_MODE=manual`, the runner calls `/v1/review/approve` after projection and before the external-model request. The approval reason is synthetic-test specific and the report records the manual review state. API keys and Authorization headers are never written.

## Flow

For each case:

1. Send synthetic raw input to the local-model simulation API.
2. Send the same synthetic input to `/v1/project`.
3. If manual review mode is enabled, approve the projected `external_view` through `/v1/review/approve`.
4. Send only `external_view` to the external-model simulation API.
5. Call `/v1/inspect-output` on the external model output.
6. Call `/v1/restore-output` to verify the local restoration path.
7. Compute privacy, utility, review-gate, and gateway report metrics.
8. Write JSON and Markdown reports.

## Output

Default output directory:

```text
data/external-api-simulation-results/
```

Files:

- `external_api_simulation_report.json`
- `external_api_simulation_report.md`
- `model_io_trace.jsonl` when `--record-model-io` is enabled.

The trace file records local model, gateway, and external model input/output bodies for synthetic runs only. It does not record API keys or Authorization headers.
