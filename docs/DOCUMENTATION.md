# Documentation Map

This repository uses English-only documentation. Keep documents concise, reproducible, and explicit about assumptions.

## Recommended Reading Order

1. Start with [WHITEPAPER.md](WHITEPAPER.md) for the goal hierarchy and trust-boundary model.
2. Continue with [ARCHITECTURE.md](ARCHITECTURE.md), [THREAT_MODEL.md](THREAT_MODEL.md), and [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md).
3. Use [API.md](API.md), [COMMANDS.md](COMMANDS.md), and [TEST_PLAN.md](TEST_PLAN.md) when changing behavior.
4. Use [ROADMAP.md](ROADMAP.md) to understand what is already implemented versus what is next.

## Research and Boundaries

| Document | Purpose |
|---|---|
| [WHITEPAPER.md](WHITEPAPER.md) | Research claim, mathematical model, system boundary, and scope |
| [THREAT_MODEL.md](THREAT_MODEL.md) | Assets, attackers, attack surfaces, assumptions, and non-goals |
| [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md) | Privacy checks, utility checks, structural fidelity, and replay |
| [EVALUATION_PLAN.md](EVALUATION_PLAN.md) | Privacy, utility, complex datasets, and quality gates |

## Engineering

| Document | Purpose |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Components, data flow, trust boundary, and module ownership |
| [API.md](API.md) | HTTP endpoints and request/response examples |
| [TECH_STACK.md](TECH_STACK.md) | Implementation stack and runtime assumptions |
| [MODEL_ADAPTERS.md](MODEL_ADAPTERS.md) | External model adapter boundary |
| [ROADMAP.md](ROADMAP.md) | Implemented 2.x slices, next delivery track, research extensions, and non-goals |

## Operation

| Document | Purpose |
|---|---|
| [COMMANDS.md](COMMANDS.md) | Development, build, run, test, and static check commands |
| [LOCAL_DEPENDENCY_CACHE.md](LOCAL_DEPENDENCY_CACHE.md) | Local dependency and build cache policy |
| [RUNTIME_ENVIRONMENT.md](RUNTIME_ENVIRONMENT.md) | Linux/container-first runtime assumptions |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Docker Compose and Kubernetes examples |
| [SECURITY_OPERATIONS.md](SECURITY_OPERATIONS.md) | Key, mapping, audit, and output inspection requirements |
| [OBSERVABILITY.md](OBSERVABILITY.md) | Tracing and OpenTelemetry samples |

## Evaluation and Community

| Document | Purpose |
|---|---|
| [TEST_PLAN.md](TEST_PLAN.md) | Local build, API, privacy, utility, audit, and deployment checks |
| [EXTERNAL_API_SIMULATION_TEST.md](EXTERNAL_API_SIMULATION_TEST.md) | Synthetic two-model external API simulation |
| [OPEN_SOURCE_RELEASE.md](OPEN_SOURCE_RELEASE.md) | Release contents, exclusions, and sensitive-data checks |
| [CONTRIBUTOR_TASKS.md](CONTRIBUTOR_TASKS.md) | Starter tasks and modular contribution areas |
| [RFC_PROCESS.md](RFC_PROCESS.md) | Design process for privacy, policy, adapter, and benchmark changes |
| [rfcs/](rfcs/) | RFC index and template |

## Maintenance Rules

- When project goals or terminology change, update [README.md](../README.md), [WHITEPAPER.md](WHITEPAPER.md), and [ROADMAP.md](ROADMAP.md) together first.
- API changes must update [API.md](API.md) and [TEST_PLAN.md](TEST_PLAN.md).
- Policy or mechanism changes must update [TECH_STACK.md](TECH_STACK.md), [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md), and sample policy files.
- Deployment variable changes must update [COMMANDS.md](COMMANDS.md), [DEPLOYMENT.md](DEPLOYMENT.md), `.env.example`, and `docker-compose.yml`.
- Trust-boundary changes must update [THREAT_MODEL.md](THREAT_MODEL.md) and [SECURITY_OPERATIONS.md](SECURITY_OPERATIONS.md).
- Community-process changes must update `CONTRIBUTING.md`, `GOVERNANCE.md`, issue templates, and the PR template.
