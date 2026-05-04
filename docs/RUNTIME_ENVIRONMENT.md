# 运行环境

## 结论

ProofGate 的目标运行环境应以 Linux 服务器和容器为主，模型接入边界应以 OpenAI-compatible HTTP API 为主。

原因是市面上常见的本地模型部署方式通常会暴露一个 HTTP 服务，而不是要求调用方直接嵌入推理引擎：

- Ollama 提供 OpenAI API 兼容能力。
- vLLM 提供 OpenAI-compatible server。
- llama.cpp 的 `llama-server` 提供 OpenAI-compatible HTTP API。
- Hugging Face Text Generation Inference 支持兼容 OpenAI Chat Completion 的 Messages API。

因此 ProofGate 不应绑定某一个本地推理框架，而应把本地模型视为一个可配置的 `base_url + api_key + model`。

## 一等支持环境

一等支持：

- Linux x86_64 服务器。
- Docker / Docker Compose。
- Kubernetes。
- OpenAI-compatible `/chat/completions` 模型端点。

开发便利支持：

- Windows PowerShell。
- 本地 Rust 开发环境。

Windows 脚本只用于开发便利，不作为生产运行假设。

## 常见本地模型端点

### Ollama

典型配置：

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="ollama"
export LOCAL_MODEL_NAME="qwen2.5:7b"
```

### vLLM

典型配置：

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:8000/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="Qwen/Qwen2.5-7B-Instruct"
```

### llama.cpp / llama-server

典型配置：

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:8081/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="local-gguf-model"
```

### Hugging Face TGI

典型配置：

```bash
export LOCAL_MODEL_BASE_URL="http://127.0.0.1:8080/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="tgi-local-model"
```

## ProofGate 运行方式

Linux 本地运行：

```bash
cd /opt/ProofGate
source ./scripts/dev-env.sh
export PROOFGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p proofgate-gateway
```

外部 API 模拟测试：

```bash
cd /opt/ProofGate
source ./scripts/dev-env.sh

export LOCAL_MODEL_BASE_URL="http://127.0.0.1:11434/v1"
export LOCAL_MODEL_API_KEY="local"
export LOCAL_MODEL_NAME="qwen2.5:7b"

export EXTERNAL_MODEL_BASE_URL="https://external-compatible-provider.example/v1"
export EXTERNAL_MODEL_API_KEY="replace-with-test-key"
export EXTERNAL_MODEL_NAME="replace-with-model"

./scripts/run-external-api-simulation.sh
```

## 设计要求

- ProofGate 不直接依赖 CUDA、llama.cpp、vLLM、Ollama 或 TGI 的进程内 API。
- ProofGate 只依赖模型服务暴露的 HTTP API。
- 本地模型端点和外部模型端点使用同一 adapter 协议。
- 本地模型模拟可以接收合成原始输入；外部模型只能接收 `external_view`。
- 生产部署优先使用容器和 Linux 服务管理，不依赖 PowerShell。
