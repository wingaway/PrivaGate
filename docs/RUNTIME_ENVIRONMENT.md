# Runtime Environment

ProofGate is designed for Linux and container-first deployment because common local model stacks such as Ollama, vLLM, llama.cpp services, and GPU inference servers are usually deployed on Linux hosts.

Windows is supported for local development and testing through PowerShell scripts.

## Linux

```bash
source ./scripts/dev-env.sh
export PROOFGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p proofgate-gateway
```

## Windows

```powershell
.\scripts\dev-env.ps1
$env:PROOFGATE_HMAC_KEY="replace-with-local-secret"
.\scripts\cargo.ps1 run -p proofgate-gateway
```

## Required Environment Variables

- `PROOFGATE_HMAC_KEY`: local HMAC tokenization key.
- `PROOFGATE_POLICY_PATH`: policy JSON path.
- `PROOFGATE_MAPPING_LOG`: local mapping JSONL path.
- `PROOFGATE_AUDIT_LOG`: local audit JSONL path.

Optional:

- `PROOFGATE_AUDIT_POSTGRES_URL`: PostgreSQL audit backend.
- `RUST_LOG`: tracing filter.
- `PROOFGATE_URL`: simulation gateway URL.

## Model Runtime Assumption

Local models should run inside the local trust boundary. External model APIs must receive only projected views.
