# Test Plan

## Local Build Gate

Run:

```bash
source ./scripts/dev-env.sh
./scripts/cargo.sh fmt --all -- --check
./scripts/cargo.sh test
./scripts/cargo.sh clippy --all-targets -- -D warnings
./scripts/cargo.sh build --release -p privagate-gateway
```

Windows:

```powershell
.\scripts\dev-env.ps1
.\scripts\cargo.ps1 fmt --all -- --check
.\scripts\cargo.ps1 test
.\scripts\cargo.ps1 clippy --all-targets -- -D warnings
.\scripts\cargo.ps1 build --release -p privagate-gateway
```

Expected result: formatting passes, tests pass, clippy has no warnings, and release build succeeds.

## API Smoke Test

Start the gateway:

```bash
export PRIVAGATE_HMAC_KEY="replace-with-local-secret"
./scripts/cargo.sh run -p privagate-gateway
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
- `task_contract_assessment` reflects the requested or default task profile.

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

## Manual Review Gate

Start the gateway with manual review enabled:

```bash
export PRIVAGATE_REVIEW_MODE=manual
./scripts/cargo.sh run -p privagate-gateway
```

Project a synthetic request. Expected result:

- `audit_summary.blocked=true`;
- `manual_review.status="pending"`;
- `manual_review.external_view_digest` equals `audit_summary.external_view_digest`.

Before approval, `/v1/model-dispatch` with the same `audit_id` must return `dispatched=false` and `blocked_by_review=true`.

Approve the projection:

```bash
curl -sS http://127.0.0.1:8080/v1/review/approve \
  -H 'Content-Type: application/json' \
  -d '{"audit_id":"<audit-id>","reviewer":"reviewer-id","reason":"synthetic projected view approved"}'
```

Expected result: `/v1/model-dispatch` is no longer blocked by the manual review gate when the dispatch request carries the same `audit_id` and unchanged `external_view` digest. A modified external view must still be blocked by digest mismatch.

## Route-Plan Validation

Call `/v1/route-plan/validate` with `examples/route-plan-request.json`.

Expected result:

- every stage includes `external_view_digest`;
- `task_contract_assessment` is present for each stage;
- `dispatch_allowed` becomes false when review, task-contract, or adapter-class checks fail;
- a local `route_plan_evidence` audit event is emitted.

## Route-Plan Execution

Start the gateway with `PRIVAGATE_MODEL_ADAPTER=dry_run` and call `/v1/route-plan/execute` with `examples/route-plan-execute-request.json`.

Expected result:

- runtime adapter capability is returned in `runtime_adapter_capabilities`;
- every executed stage returns `dispatch_response`;
- `stop_on_block=true` halts later stages after the first blocked or non-dispatched stage;
- a local `route_plan_evidence` audit event with execution outcome is emitted.

## Shard-Plan Validation

Call `/v1/shard-plan/validate` with `examples/shard-plan-request.json`.

Expected result:

- route-plan checks still apply to every stage;
- `local_aggregation_summary` reports shard completeness and missing-group issues;
- promotion readiness appears only as aggregation-rule assessment, not as a materialized follow-up view;
- a local `shard_plan_evidence` audit event is emitted.

## Shard-Plan Execution and Local Aggregation

Start the gateway with `PRIVAGATE_MODEL_ADAPTER=dry_run` and call `/v1/shard-plan/execute` with `examples/shard-plan-execute-request.json` or `examples/shard-plan-promote-request.json`.

Expected result:

- complete shard groups receive `local_aggregation_digest`;
- non-`digest_only` strategies emit `local_only_output` and `local_only_output_digest`;
- promotion candidates appear only when aggregation rules and task contracts allow them;
- `promotion.utility_assessment` reflects required-field and optional structural utility checks for the follow-up task profile;
- a local `shard_plan_evidence` audit event with execution details is emitted.

## Promotion Binding

Call `/v1/shard-plan/bind-promotion` using shard execution output plus `group_id="claims"`.

Expected result:

- replay verification rejects inconsistent or tampered shard execution evidence;
- `binding_created=true` creates a new follow-up `audit_id`, `audit_summary`, and `external_view`;
- when `PRIVAGATE_REVIEW_MODE=manual`, the new binding also creates a pending `manual_review` record;
- `binding_created=false` when promotion utility verification fails or dispatch-output digests do not match;
- a local `promotion_binding_evidence` audit event is emitted.

## Follow-Up Utility Gate Outcomes

Run at least one positive and one negative promotion case.

Expected result:

- a valid follow-up candidate preserves the required fields or structural constraints declared by the follow-up task profile;
- a failing candidate keeps `promotion.utility_assessment` for diagnosis but does not create a reusable binding;
- task-profile-specific `promotion_utility` policy rules are honored even when request-level promotion verification is minimal.

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
- route or shard follow-up tests using dry-run adapters keep raw-value echo count at zero as well;
- reports are written under ignored `data/`.

## Release Gate

Run:

```bash
./scripts/prepublish-check.sh
```

Expected result: no potential secret, real-data artifact, or required-file gap is found.
