# Deployment

## Docker Compose

Linux:

```bash
source ./scripts/dev-env.sh
export PROOFGATE_HMAC_KEY="replace-with-deployment-secret"
docker compose config
docker compose up --build
```

Windows:

```powershell
.\scripts\dev-env.ps1
$env:PROOFGATE_HMAC_KEY="replace-with-deployment-secret"
docker compose config
docker compose up --build
```

## Kubernetes

The sample manifest is located at:

```text
deploy/kubernetes/proofgate-gateway.yaml
```

Before use, replace secrets, storage settings, image names, and ingress rules with environment-specific values.

## Runtime Requirements

- Rust build output or container image.
- Local policy file.
- Local HMAC key.
- Protected local mapping storage.
- Append-only audit storage.

## Deployment Rule

External model integrations must receive only `external_view`. Raw input, local mappings, and keys must remain inside the local trust boundary.
