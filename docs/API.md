# API Reference

All examples assume the gateway is running at `http://127.0.0.1:8080`. Requests and responses are JSON unless noted.

## POST /v1/project

Project raw input into an `external_view` and return privacy, utility, and audit reports.

Request:

```json
{
  "task_profile": "contract_risk_review",
  "content_type": "json",
  "payload": {}
}
```

Supported `content_type` values:

- `json`
- `csv_rows`
- `text`

Response:

```json
{
  "external_view": {},
  "privacy_report": {},
  "utility_report": {},
  "audit_summary": {},
  "task_contract_assessment": {
    "task_profile": "contract_risk_review",
    "dispatch_allowed": true,
    "issues": []
  },
  "manual_review": {
    "required": true,
    "audit_id": "00000000-0000-0000-0000-000000000000",
    "external_view_digest": "sha256:...",
    "status": "pending",
    "reviewer": null,
    "reason": "manual review required before external dispatch"
  }
}
```

Local token mappings are never returned. They are written to the local JSONL file configured by `PRIVAGATE_MAPPING_LOG`. Audit summaries and verification results are written to `PRIVAGATE_AUDIT_LOG`.

`task_profile` is optional. When omitted, PrivaGate uses the default `task_profile` declared by the loaded policy. When supplied, PrivaGate evaluates the projection and task-contract checks against the requested profile.

`manual_review` is returned only when `PRIVAGATE_REVIEW_MODE=manual`. In that mode `audit_summary.blocked=true` until the projected view is approved. The approval is bound to `audit_id` and `external_view_digest`.

`task_contract_assessment` tells the caller whether the projected result is eligible for downstream dispatch under the current task contract. If the task requires a field that is marked `local_only`, `dispatch_allowed=false`, `audit_summary.blocked=true`, and PrivaGate will later block `/v1/model-dispatch` for the same task profile.

When a task contract blocks projection from continuing to downstream dispatch, the local audit log records a `task_contract_gate` event for the same `audit_id`. If a caller later hits `/v1/model-dispatch` with a blocked task profile anyway, PrivaGate records a `blocked_dispatch` task-contract event there as well.

## POST /v1/statistics

Run local differential-privacy statistics declared by policy. Row-level data is not sent externally.

Current mechanisms:

- `laplace_count`
- `laplace_histogram`
- `laplace_mean`

Request:

```json
{
  "content_type": "json",
  "payload": {
    "contracts": [],
    "events": [],
    "payments": []
  }
}
```

Response:

```json
{
  "input_digest": "sha256:...",
  "privacy_budget": {
    "epsilon": 2.25,
    "delta": 0.0,
    "consumed": true
  },
  "results": [],
  "verification_results": []
}
```

## POST /v1/inspect-output

Inspect external model output against the local mapping for an `audit_id`.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "output": "model output text"
}
```

Response:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "passed": true,
  "unauthorized_sensitive_output_count": 0,
  "findings": []
}
```

## POST /v1/restore-output

Restore authorized tokens in model output using the local mapping log. This endpoint does not call any external model.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "output": "model output with <PERSON_xxx>"
}
```

Response:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "restored_output": "model output with original value",
  "replacements": 1
}
```

## POST /v1/review/status

Read the manual review state for a projected view.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000"
}
```

Response:

```json
{
  "required": true,
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "external_view_digest": "sha256:...",
  "status": "pending",
  "reviewer": null,
  "reason": "manual review required before external dispatch",
  "created_at": "2026-05-04T00:00:00Z",
  "updated_at": "2026-05-04T00:00:00Z"
}
```

## POST /v1/review/approve

Approve a projected view after human review. Approval does not expose raw mappings and does not call an external model.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "reviewer": "reviewer-id",
  "reason": "projected view inspected and approved"
}
```

Response: same shape as `/v1/review/status`, with `status="approved"`.

## POST /v1/review/reject

Reject a projected view. A rejected view cannot pass `/v1/model-dispatch` while manual review mode is enabled.

Request:

```json
{
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "reviewer": "reviewer-id",
  "reason": "projection contains unsafe context"
}
```

Response: same shape as `/v1/review/status`, with `status="rejected"`.

Manual review state is loaded from the configured review store. The default store is `PRIVAGATE_REVIEW_LOG`. `PRIVAGATE_REVIEW_POSTGRES_URL` can be used for shared or restart-safe review workflows.

## POST /v1/rag/project-chunks

Project RAG chunks before indexing or before external-context use.

Request:

```json
{
  "task_profile": "rag_privacy_projection",
  "chunks": [
    {
      "chunk_id": "chunk-1",
      "source_uri": "local://doc-1",
      "content_type": "text",
      "payload": "please contact +1 650-555-0142"
    }
  ]
}
```

Response:

```json
{
  "chunks": [
    {
      "chunk_id": "chunk-1",
      "source_uri": "local://doc-1",
      "external_view_digest": "sha256:...",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "external_view": {},
      "privacy_passed": true,
      "utility_passed": true,
      "task_contract_assessment": {
        "task_profile": "rag_privacy_projection",
        "dispatch_allowed": true,
        "issues": []
      },
      "manual_review": {
        "required": true,
        "audit_id": "00000000-0000-0000-0000-000000000000",
        "external_view_digest": "sha256:...",
        "status": "pending"
      }
    }
  ]
}
```

`task_profile` is optional here as well. When supplied, the same task-contract checks used by `/v1/project` are applied to every chunk projection. If a chunk is blocked by task contract, the response returns `task_contract_assessment.dispatch_allowed=false`, `audit_summary.blocked=true` is reflected through the stored audit summary for that chunk, and the local audit log records a `task_contract_gate` event.

## POST /v1/tool/inspect

Inspect agent tool input and output for unauthorized sensitive values associated with previous audit IDs.

Request:

```json
{
  "tool_name": "database.query",
  "audit_ids": ["00000000-0000-0000-0000-000000000000"],
  "input": "tool input text",
  "output": "tool output text"
}
```

## POST /v1/session/risk

Compute session-level exposure and accumulated differential-privacy budget.

Request:

```json
{
  "session_id": "session-1",
  "risk_bound": 5.0,
  "events": [
    {
      "external_view_digest": "sha256:...",
      "privacy_budget_epsilon": 0.5
    }
  ]
}
```

## POST /v1/model-dispatch

Reserved model adapter boundary. The default `disabled` adapter does not call model providers. `PRIVAGATE_MODEL_ADAPTER=dry_run` can be used to test dispatch readiness against projected views without calling an external provider.

Request:

```json
{
  "provider": "reserved",
  "task_profile": "contract_risk_review",
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "external_view": {
    "content_type": "json",
    "payload": {}
  }
}
```

Response:

```json
{
  "provider": "reserved",
  "dispatched": false,
  "status": "external model adapter is reserved but not configured for task_profile=contract_risk_review",
  "output": null,
  "audit_id": "00000000-0000-0000-0000-000000000000",
  "external_view_digest": "sha256:...",
  "blocked_by_review": false,
  "blocked_by_policy": false,
  "adapter_capabilities": {
    "adapter_class": "reserved",
    "accepts_external_view_only": true,
    "supports_digest_binding": true,
    "supports_manual_review_gate": true
  }
}
```

When `PRIVAGATE_REVIEW_MODE=manual`, `/v1/model-dispatch` requires an approved review record for the supplied `audit_id`. It recomputes the digest of the supplied `external_view` and blocks dispatch if the digest differs from the reviewed digest.

When the policy declares a task contract for the supplied `task_profile`, `/v1/model-dispatch` also checks whether the task requires fields that the policy marks as `local_only`. In that case dispatch is blocked before the adapter boundary because the task would require data that is not allowed to cross the trusted boundary.

If the task contract declares `allowed_adapter_classes`, `/v1/model-dispatch` also compares the adapter's capability metadata against that list. A mismatch blocks dispatch before the adapter call and records an `adapter_capability_gate` audit event.

## POST /v1/route-plan/validate

Validate a multi-stage route or shard plan without dispatching it. This endpoint computes per-stage digests, checks task contracts, checks declared adapter classes, applies the manual review gate, and records local route-plan evidence.

Request:

```json
{
  "route_id": "route-001",
  "aggregation_strategy": "merge_stage_outputs",
  "residual_risk_notes": [
    "synthetic example"
  ],
  "stages": [
    {
      "stage_id": "stage-1",
      "provider": "dry-run",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "external_view": {
        "content_type": "json",
        "payload": {}
      }
    },
    {
      "stage_id": "stage-2",
      "provider": "dry-run",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "shard_group": "claims",
      "shard_id": "shard-b",
      "external_view": {
        "content_type": "json",
        "payload": {}
      }
    }
  ]
}
```

Response:

```json
{
  "route_id": "route-001",
  "aggregation_strategy": "merge_stage_outputs",
  "residual_risk_notes": [
    "synthetic example"
  ],
  "dispatch_allowed": false,
  "stages": [
    {
      "stage_id": "stage-1",
      "provider": "dry-run",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "external_view_digest": "sha256:...",
      "blocked_by_policy": false,
      "blocked_by_review": true,
      "dispatch_allowed": false,
      "issues": [
        "audit_id=00000000-0000-0000-0000-000000000000 is pending manual review"
      ],
      "task_contract_assessment": {
        "task_profile": "contract_risk_review",
        "dispatch_allowed": true,
        "issues": []
      }
    }
  ]
}
```

`adapter_class` is declared per stage so a route plan can be checked before a concrete adapter instance exists. This endpoint records a `route_plan_evidence` audit row with the route ID, stage digests, provider identities, aggregation strategy, residual-risk notes, and validation outcome.

`shard_group` and `shard_id` may also be supplied per stage. They are optional for route-plan endpoints and are carried into the local evidence so shard-aware workflows can be introduced incrementally.

## POST /v1/route-plan/execute

Execute a validated route plan stage by stage through the configured adapter boundary. This endpoint is intended for bounded experimentation. It reuses the same checks as `/v1/route-plan/validate`, then compares each stage's declared `adapter_class` with the runtime adapter capability and records execution evidence locally.

Use `PRIVAGATE_MODEL_ADAPTER=dry_run` to exercise this path without contacting a real provider.

Request:

```json
{
  "route_id": "route-exec-001",
  "aggregation_strategy": "merge_stage_outputs",
  "residual_risk_notes": [
    "synthetic staged execution example"
  ],
  "stop_on_block": true,
  "stages": [
    {
      "stage_id": "stage-1",
      "provider": "dry-run",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "external_view": {
        "content_type": "json",
        "payload": {}
      }
    }
  ]
}
```

Response:

```json
{
  "route_id": "route-exec-001",
  "aggregation_strategy": "merge_stage_outputs",
  "residual_risk_notes": [
    "synthetic staged execution example"
  ],
  "dispatch_allowed": true,
  "stop_on_block": true,
  "halted": false,
  "executed_stage_count": 1,
  "dispatched_stage_count": 1,
  "all_stages_dispatched": true,
  "runtime_adapter_capabilities": {
    "adapter_class": "local_private",
    "accepts_external_view_only": true,
    "supports_digest_binding": true,
    "supports_manual_review_gate": true
  },
  "stages": [
    {
      "stage_id": "stage-1",
      "provider": "dry-run",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "external_view_digest": "sha256:...",
      "blocked_by_policy": false,
      "blocked_by_review": false,
      "dispatch_allowed": true,
      "issues": [],
      "task_contract_assessment": {
        "task_profile": "contract_risk_review",
        "dispatch_allowed": true,
        "issues": []
      },
      "executed": true,
      "runtime_issues": [],
      "dispatch_response": {
        "provider": "dry-run",
        "dispatched": true,
        "status": "dry_run adapter accepted projected request"
      }
    }
  ]
}
```

When `stop_on_block=true`, the gateway stops later stages after the first blocked or non-dispatched stage and marks the remaining stages with `skipped_reason`.

This endpoint records a `route_plan_evidence` row with `event="executed"`, the runtime adapter capability, stage execution results, and the final halt status.

## POST /v1/shard-plan/validate

Validate a shard-aware plan and compute local aggregation readiness without dispatching it. This endpoint reuses route-plan validation, then applies shard-group rules such as required shard metadata, expected groups, minimum shard counts, and optional provider-diversity requirements.

Request:

```json
{
  "route_id": "shard-plan-001",
  "aggregation_strategy": "local_merge",
  "residual_risk_notes": [
    "synthetic shard validation example"
  ],
  "aggregation_rules": {
    "require_shard_metadata": true,
    "require_distinct_providers": false,
    "strategy": "collect_outputs",
    "promotion": {
      "mode": "external_view_candidate",
      "candidate_task_profile": "contract_risk_review"
    },
    "expected_groups": [
      {
        "group_id": "claims",
        "min_shards": 2,
        "strategy": "collect_outputs"
      }
    ]
  },
  "stages": [
    {
      "stage_id": "stage-1",
      "provider": "dry-run-a",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "shard_group": "claims",
      "shard_id": "shard-a",
      "external_view": {
        "content_type": "json",
        "payload": {}
      }
    }
  ]
}
```

Response:

```json
{
  "route_plan": {
    "route_id": "shard-plan-001",
    "aggregation_strategy": "local_merge",
    "dispatch_allowed": true
  },
  "aggregation_rules": {
    "require_shard_metadata": true,
    "require_distinct_providers": false,
    "strategy": "collect_outputs",
    "expected_groups": [
      {
        "group_id": "claims",
        "min_shards": 2,
        "strategy": "collect_outputs"
      }
    ]
  },
  "local_aggregation_summary": {
    "aggregation_strategy": "local_merge",
    "ready_for_local_aggregation": false,
    "issue_count": 1,
    "issues": [
      "shard_group=claims expected at least 2 shards but observed 1"
    ],
    "groups": [
      {
        "group_id": "claims",
        "aggregation_strategy": "collect_outputs",
        "expected_min_shards": 2,
        "observed_stage_count": 1
      }
    ]
  }
}
```

Optional `aggregation_rules.promotion` rules let the gateway assess whether a future local aggregation output may later become a candidate `external_view`. Validation cannot materialize that candidate yet because no adapter outputs exist, so a configured promotion rule appears only as readiness assessment after shard execution.

This endpoint records a `shard_plan_evidence` audit row with the route-plan assessment, aggregation rules, and local aggregation summary.

## POST /v1/shard-plan/execute

Execute a shard-aware staged plan through the configured adapter and emit local aggregation evidence. This endpoint reuses route-plan execution, then summarizes shard-group completeness and computes local aggregation digests from stage bindings and stage output digests.

Use `PRIVAGATE_MODEL_ADAPTER=dry_run` to exercise this path without contacting a real provider.

Request:

```json
{
  "route_id": "shard-exec-001",
  "aggregation_strategy": "local_merge",
  "residual_risk_notes": [
    "synthetic shard execution example"
  ],
  "stop_on_block": true,
  "aggregation_rules": {
    "require_shard_metadata": true,
    "require_distinct_providers": false,
    "strategy": "collect_outputs",
    "promotion": {
      "mode": "external_view_candidate",
      "candidate_task_profile": "follow_up_summary",
      "utility_verification": {
        "require_required_fields": true,
        "verify_constraint_results": false
      },
      "allowed_content_types": [
        "json"
      ],
      "max_array_items": 4
    },
    "expected_groups": [
      {
        "group_id": "claims",
        "min_shards": 2,
        "strategy": "collect_outputs"
      }
    ]
  },
  "stages": [
    {
      "stage_id": "stage-1",
      "provider": "dry-run",
      "task_profile": "contract_risk_review",
      "adapter_class": "local_private",
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "shard_group": "claims",
      "shard_id": "shard-a",
      "external_view": {
        "content_type": "json",
        "payload": {}
      }
    }
  ]
}
```

Response:

```json
{
  "route_plan_execution": {
    "route_id": "shard-exec-001",
    "dispatch_allowed": true,
    "halted": false,
    "executed_stage_count": 2,
    "dispatched_stage_count": 2
  },
  "aggregation_rules": {
    "require_shard_metadata": true,
    "require_distinct_providers": false,
    "strategy": "collect_outputs",
    "expected_groups": [
      {
        "group_id": "claims",
        "min_shards": 2,
        "strategy": "collect_outputs"
      }
    ]
  },
  "local_aggregation_summary": {
    "aggregation_strategy": "local_merge",
    "ready_for_local_aggregation": true,
    "issue_count": 0,
    "issues": [],
    "groups": [
      {
        "group_id": "claims",
        "aggregation_strategy": "collect_outputs",
        "observed_stage_count": 2,
        "dispatched_stage_count": 2,
        "all_stage_outputs_bound": true,
        "local_aggregation_digest": "sha256:...",
        "local_only_output_digest": "sha256:...",
        "local_only_output": [
          {
            "stage_id": "stage-1",
            "shard_id": "shard-a",
            "provider": "dry-run",
            "output": {}
          },
          {
            "stage_id": "stage-2",
            "shard_id": "shard-b",
            "provider": "dry-run",
            "output": {}
          }
        ],
        "promotion": {
          "mode": "external_view_candidate",
          "promotion_allowed": true,
          "ready_for_follow_up_route": true,
          "candidate_task_profile": "follow_up_summary",
          "candidate_content_type": "json",
          "task_contract_assessment": {
            "task_profile": "follow_up_summary",
            "dispatch_allowed": true,
            "issues": []
          },
          "utility_assessment": {
            "task_profile": "follow_up_summary",
            "verification_passed": true,
            "required_field_presence": {
              "required": 0,
              "preserved": 0,
              "ratio": 1.0
            },
            "missing_required_fields": [],
            "constraint_results": [],
            "issues": []
          },
          "issues": [],
          "external_view_candidate_digest": "sha256:...",
          "external_view_candidate": {
            "content_type": "json",
            "payload": [
              {
                "stage_id": "stage-1",
                "shard_id": "shard-a",
                "provider": "dry-run",
                "output": {}
              },
              {
                "stage_id": "stage-2",
                "shard_id": "shard-b",
                "provider": "dry-run",
                "output": {}
              }
            ]
          }
        }
      }
    ]
  }
}
```

Supported aggregation strategies are:

- `digest_only`
- `collect_outputs`
- `json_object_by_shard`
- `text_concatenate`

The global `aggregation_rules.strategy` can be overridden per expected group. Aggregated outputs are local-only outputs: they are returned by the local gateway for local workflows and evidence, but they are not intended to cross the less-trusted boundary automatically.

When all shard rules pass and every stage in a group dispatches successfully, `local_aggregation_summary.ready_for_local_aggregation=true`. Each complete group receives a stable `local_aggregation_digest` for stage-binding evidence, and non-`digest_only` strategies may also emit `local_only_output` plus `local_only_output_digest`. These digests are intended for local replay and aggregation evidence; they are not a claim that the shard outputs are semantically correct.

`aggregation_rules.promotion` is optional and conservative by default. When `mode=external_view_candidate`, the gateway may evaluate whether a local aggregation output can be wrapped as a candidate `external_view` for a follow-up task profile. Promotion rules can restrict content type and output size through `allowed_content_types`, `max_serialized_bytes`, `max_text_chars`, `max_array_items`, and `max_object_keys`.

`aggregation_rules.promotion.utility_verification` controls the local follow-up utility gate. By default, PrivaGate checks whether the promoted view still contains the required fields declared by the follow-up task contract. Callers may also enable `verify_constraint_results` to run the policy's structural utility checks against the promoted output before binding it for later dispatch.

Task contracts may also declare `promotion_utility` in policy. Those task-profile-specific rules are merged into the same local gate, so a follow-up task can require checks such as `relation_preservation` even when the request-level `utility_verification.verify_constraint_results=false`.

Even when promotion succeeds, the candidate is still local evidence. PrivaGate does not auto-dispatch it. In manual review mode, `promotion_allowed=true` may still produce `ready_for_follow_up_route=false` because aggregated candidates do not yet carry a projection-backed `audit_id`.

This endpoint records a `shard_plan_evidence` row with `event="executed"`, the route-plan execution result, the aggregation rules, and the local aggregation summary.

## POST /v1/shard-plan/bind-promotion

Bind a promoted local aggregation candidate to a new local `audit_id` so it can participate in a later route plan. This endpoint does not re-dispatch shard stages. Instead, it re-evaluates the supplied shard execution evidence, verifies dispatch-output digests, selects the named shard group, and if promotion is allowed, creates a new local follow-up view binding.

Request:

```json
{
  "route_plan_execution": {
    "route_id": "shard-promote-001",
    "aggregation_strategy": "local_merge",
    "residual_risk_notes": [
      "synthetic local aggregation promotion example"
    ],
    "dispatch_allowed": true,
    "stop_on_block": true,
    "halted": false,
    "executed_stage_count": 2,
    "dispatched_stage_count": 2,
    "all_stages_dispatched": true,
    "runtime_adapter_capabilities": {
      "adapter_class": "local_private",
      "accepts_external_view_only": true,
      "supports_digest_binding": true,
      "supports_manual_review_gate": true
    },
    "stages": []
  },
  "aggregation_rules": {
    "require_shard_metadata": true,
    "require_distinct_providers": false,
    "strategy": "json_object_by_shard",
    "promotion": {
      "mode": "external_view_candidate",
      "candidate_task_profile": "follow_up_summary",
      "utility_verification": {
        "require_required_fields": true,
        "verify_constraint_results": false
      }
    },
    "expected_groups": [
      {
        "group_id": "claims",
        "min_shards": 2,
        "strategy": "json_object_by_shard"
      }
    ]
  },
  "group_id": "claims"
}
```

Response:

```json
{
  "route_id": "shard-promote-001",
  "group_id": "claims",
  "binding_created": true,
  "ready_for_follow_up_route": false,
  "issues": [],
  "source_local_aggregation_digest": "sha256:...",
  "source_local_only_output_digest": "sha256:...",
  "promotion": {
    "mode": "external_view_candidate",
    "promotion_allowed": true,
    "ready_for_follow_up_route": true,
    "candidate_task_profile": "follow_up_summary",
    "candidate_content_type": "json",
    "task_contract_assessment": {
      "task_profile": "follow_up_summary",
      "dispatch_allowed": true,
      "issues": []
    },
    "utility_assessment": {
      "task_profile": "follow_up_summary",
      "verification_passed": true,
      "required_field_presence": {
        "required": 0,
        "preserved": 0,
        "ratio": 1.0
      },
      "missing_required_fields": [],
      "constraint_results": [],
      "issues": []
    },
    "issues": [],
    "external_view_candidate_digest": "sha256:...",
    "external_view_candidate": {
      "content_type": "json",
      "payload": {}
    }
  },
  "follow_up_binding": {
    "task_profile": "follow_up_summary",
    "audit_summary": {
      "audit_id": "00000000-0000-0000-0000-000000000000",
      "input_digest": "sha256:...",
      "external_view_digest": "sha256:...",
      "policy_version": "2026.05.03-production-sample",
      "blocked": true
    },
    "external_view": {
      "content_type": "json",
      "payload": {}
    },
    "utility_assessment": {
      "task_profile": "follow_up_summary",
      "verification_passed": true,
      "required_field_presence": {
        "required": 0,
        "preserved": 0,
        "ratio": 1.0
      },
      "missing_required_fields": [],
      "constraint_results": [],
      "issues": []
    }
  },
  "manual_review": {
    "required": true,
    "audit_id": "00000000-0000-0000-0000-000000000000",
    "external_view_digest": "sha256:...",
    "status": "pending",
    "reviewer": null,
    "reason": "manual review required before follow-up dispatch of promoted aggregation candidate"
  }
}
```

If `binding_created=true`, the response contains a new local `audit_summary` and `external_view` that can be used in a later `/v1/route-plan/validate` or `/v1/route-plan/execute` request.

If local follow-up utility verification fails, the response keeps `promotion.utility_assessment` for diagnosis but returns `binding_created=false`. This is the guard that prevents a promoted aggregation result from becoming a reusable follow-up view when required fields or explicitly requested structural constraints are no longer preserved.

When `PRIVAGATE_REVIEW_MODE=manual`, binding creates a pending review record for the new follow-up view. In that case the binding is valid but `ready_for_follow_up_route=false` until `/v1/review/approve` approves the returned `audit_id` and digest.

This endpoint refuses to bind if the replayed shard execution evidence is inconsistent. For example, if a stage output is present but its `dispatch_output_digest` is missing or incorrect, the response returns `binding_created=false` and explains the mismatch in `issues`.

This endpoint records a `promotion_binding_evidence` audit row with `event="bound"` or `event="blocked"`.
