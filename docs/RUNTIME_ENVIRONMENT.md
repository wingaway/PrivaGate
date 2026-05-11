# Runtime Environment

PrivaGate is designed for Linux and container-first deployment because common local privacy infrastructure, model-serving stacks, and AI runtime components are usually deployed on Linux hosts.

Windows is supported for local development and testing through PowerShell scripts.

## Linux

```bash
source ./scripts/dev-env.sh
export PRIVAGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p privagate-gateway
```

## Windows

```powershell
.\scripts\dev-env.ps1
$env:PRIVAGATE_HMAC_KEY="replace-with-local-secret"
.\scripts\cargo.ps1 run -p privagate-gateway
```

## Required Environment Variables

- `PRIVAGATE_HMAC_KEY`: local HMAC tokenization key.
- `PRIVAGATE_POLICY_PATH`: policy JSON path.
- `PRIVAGATE_MAPPING_LOG`: local mapping JSONL path.
- `PRIVAGATE_AUDIT_LOG`: local audit JSONL path.

Optional:

- `PRIVAGATE_AUDIT_POSTGRES_URL`: PostgreSQL audit backend.
- `PRIVAGATE_REVIEW_LOG`: JSONL-backed manual review store when PostgreSQL is not used.
- `PRIVAGATE_REVIEW_POSTGRES_URL`: dedicated PostgreSQL manual review backend. When unset, the gateway reuses `PRIVAGATE_AUDIT_POSTGRES_URL` if that audit backend is configured.
- `PRIVAGATE_REVIEW_MODE`: `off` by default; set to `manual` to require human approval before model dispatch.
- `PRIVAGATE_MODEL_ADAPTER`: `disabled` by default; set to `dry_run` to exercise the dispatch boundary without calling a real model.
- `RUST_LOG`: tracing filter.
- `PRIVAGATE_URL`: simulation gateway URL.

## Manual Review Runtime

With `PRIVAGATE_REVIEW_MODE=manual`, every projection creates a pending review record in the configured review store. The default store is `data/manual-review.jsonl`. PostgreSQL can be used for durable shared-state review workflows.

External dispatch through `/v1/model-dispatch` is blocked until `/v1/review/approve` approves the same `audit_id` and `external_view_digest`. This gate survives gateway restarts as long as the review store path or database is preserved.

## Model Runtime Assumption

Local models should run inside the local trust boundary. Less-trusted external systems, including model APIs, should receive only projected views.
