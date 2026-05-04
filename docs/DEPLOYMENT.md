# 部署

## 本地 Docker Compose

Linux:

```bash
cd /opt/ProofGate
export PROOFGATE_HMAC_KEY="replace-with-local-secret"
docker compose up --build
```

Windows PowerShell:

```powershell
Set-Location E:\CodeHub\ProofGate
$env:PROOFGATE_HMAC_KEY="replace-with-local-secret"
docker compose up --build
```

项目 compose 文件将以下路径映射到容器内：

- `./config -> /app/config`
- `./data -> /app/data`
- `./.cargo-home -> /workspace/.cargo-home`
- `./target -> /workspace/target`

Windows 使用 Docker Desktop 时，Docker Desktop 自身的数据根目录需要在 Docker Desktop 设置中迁移到非 C 盘位置。

配置检查：

```bash
export PROOFGATE_HMAC_KEY="compose-static-check"
docker compose config
```

Windows PowerShell 等价命令：

```powershell
$env:PROOFGATE_HMAC_KEY="compose-static-check"
docker compose config
```

## Kubernetes

样例文件：

```text
deploy/kubernetes/proofgate-gateway.yaml
```

部署：

```bash
kubectl apply -f ./deploy/kubernetes/proofgate-gateway.yaml
```

该样例使用：

- `Secret` 保存 HMAC key。
- `ConfigMap` 挂载策略文件。
- `PersistentVolumeClaim` 保存本地映射和审计日志。
- readiness / liveness probe 检查 `/healthz`。

生产环境需要替换镜像、Secret、策略内容和 PVC storage class。

本机无 Kubernetes API server 时，`kubectl apply --dry-run=client` 仍可能尝试 discovery 并失败。实际集群中应执行：

```bash
kubectl apply --dry-run=server -f ./deploy/kubernetes/proofgate-gateway.yaml
```
