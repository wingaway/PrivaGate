# Deployment

## Docker Compose

Linux:

```bash
source ./scripts/dev-env.sh
export PRIVAGATE_HMAC_KEY="replace-with-deployment-secret"
export PRIVAGATE_REVIEW_MODE="manual"
docker compose config
docker compose up --build
```

Windows:

```powershell
.\scripts\dev-env.ps1
$env:PRIVAGATE_HMAC_KEY="replace-with-deployment-secret"
$env:PRIVAGATE_REVIEW_MODE="manual"
docker compose config
docker compose up --build
```

## Kubernetes

The sample manifest is located at:

```text
deploy/kubernetes/privagate-gateway.yaml
```

Before use, replace secrets, storage settings, image names, and ingress rules with environment-specific values.

## Runtime Requirements

- Rust build output or container image.
- Local policy file.
- Local HMAC key.
- Protected local mapping storage.
- Append-only audit storage.
- Optional manual review gate with `PRIVAGATE_REVIEW_MODE=manual`.

## Deployment Rule

External model integrations must receive only `external_view`. Raw input, local mappings, and keys must remain inside the local trust boundary.

For deployments that include an external model adapter, enable manual review mode when policy requires human approval before dispatch. The approval record is bound to `audit_id` and `external_view_digest`.
