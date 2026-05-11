mod model_adapter;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use privagate_core::{
    compute_statistics, process_document, GatewayInput, GatewayOutput, Policy, StatisticOutput,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::model_adapter::{
    AdapterCapabilities, DisabledModelAdapter, DryRunAdapter, ModelAdapter, ModelDispatchRequest,
    ModelDispatchResponse,
};

const SERVICE_NAME: &str = "privagate-gateway";
const CREATE_POSTGRES_AUDIT_TABLE_SQL: &str = "create table if not exists privagate_audit_log (
    id bigserial primary key,
    created_at timestamptz not null default now(),
    record jsonb not null
);";
const INSERT_POSTGRES_AUDIT_ROW_SQL: &str = "insert into privagate_audit_log (record) values ($1)";
const CREATE_POSTGRES_MANUAL_REVIEW_TABLE_SQL: &str =
    "create table if not exists privagate_manual_review (
    audit_id text primary key,
    external_view_digest text not null,
    status text not null,
    reviewer text null,
    reason text null,
    created_at text not null,
    updated_at text not null
);";
const UPSERT_POSTGRES_MANUAL_REVIEW_SQL: &str = "insert into privagate_manual_review (
    audit_id,
    external_view_digest,
    status,
    reviewer,
    reason,
    created_at,
    updated_at
) values ($1, $2, $3, $4, $5, $6, $7)
on conflict (audit_id) do update set
    external_view_digest = excluded.external_view_digest,
    status = excluded.status,
    reviewer = excluded.reviewer,
    reason = excluded.reason,
    created_at = excluded.created_at,
    updated_at = excluded.updated_at;";
const SELECT_POSTGRES_MANUAL_REVIEW_SQL: &str = "select
    audit_id,
    external_view_digest,
    status,
    reviewer,
    reason,
    created_at,
    updated_at
from privagate_manual_review
where audit_id = $1";

#[derive(Clone)]
struct AppState {
    policy: Arc<Policy>,
    hmac_key: Arc<Vec<u8>>,
    mapping_log: Arc<Mutex<MappingLog>>,
    audit_sink: Arc<Mutex<AuditSink>>,
    model_adapter: Arc<dyn ModelAdapter>,
    review_mode: ReviewMode,
    review_store: Arc<ReviewStore>,
}

struct MappingLog {
    path: PathBuf,
}

enum AuditSink {
    Jsonl { path: PathBuf },
    Postgres { connection_string: String },
}

#[derive(Clone)]
enum ReviewStore {
    Jsonl { path: PathBuf },
    Postgres { connection_string: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewMode {
    Off,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReviewRecord {
    audit_id: Uuid,
    external_view_digest: String,
    status: ReviewStatus,
    reviewer: Option<String>,
    reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let state = load_state()?;
    let bind = env::var("PRIVAGATE_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = bind.parse().context("invalid PRIVAGATE_BIND")?;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/project", post(project))
        .route("/v1/statistics", post(statistics))
        .route("/v1/rag/project-chunks", post(project_rag_chunks))
        .route("/v1/tool/inspect", post(inspect_tool_io))
        .route("/v1/session/risk", post(session_risk))
        .route("/v1/inspect-output", post(inspect_output))
        .route("/v1/restore-output", post(restore_output))
        .route("/v1/review/status", post(review_status))
        .route("/v1/review/approve", post(review_approve))
        .route("/v1/review/reject", post(review_reject))
        .route("/v1/model-dispatch", post(model_dispatch))
        .route("/v1/route-plan/validate", post(validate_route_plan))
        .route("/v1/route-plan/execute", post(execute_route_plan))
        .route("/v1/shard-plan/validate", post(validate_shard_plan))
        .route("/v1/shard-plan/execute", post(execute_shard_plan))
        .route("/v1/shard-plan/bind-promotion", post(bind_shard_promotion))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "PrivaGate gateway listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn project_rag_chunks(
    State(state): State<AppState>,
    Json(input): Json<RagChunkProjectRequest>,
) -> Result<Json<RagChunkProjectResponse>, ApiError> {
    let effective_policy = project_policy(&state.policy, input.task_profile.as_deref());
    let mut chunks = Vec::new();
    for chunk in input.chunks {
        let gateway_input = GatewayInput {
            content_type: chunk.content_type,
            payload: chunk.payload,
        };
        let mut output = process_document(gateway_input, &effective_policy, &state.hmac_key)?;
        let task_contract_assessment =
            assess_task_contract(&state.policy, &output.utility_report.task_profile);
        let manual_review =
            apply_projection_dispatch_controls(&state, &mut output, &task_contract_assessment)
                .await?;
        persist_local_mappings(&state, &output)?;
        persist_audit_summary(&state, &output, Some(&task_contract_assessment)).await?;
        persist_projection_control_events(
            &state,
            &output,
            &task_contract_assessment,
            manual_review.as_ref(),
        )
        .await?;
        chunks.push(RagChunkProjection {
            chunk_id: chunk.chunk_id,
            source_uri: chunk.source_uri,
            external_view_digest: output.audit_summary.external_view_digest.clone(),
            audit_id: output.audit_summary.audit_id,
            external_view: output.external_view,
            privacy_passed: output
                .privacy_report
                .verification_results
                .iter()
                .all(|result| result.passed),
            utility_passed: output
                .utility_report
                .constraint_results
                .iter()
                .all(|result| result.passed),
            manual_review,
            task_contract_assessment,
        });
    }

    Ok(Json(RagChunkProjectResponse { chunks }))
}

async fn inspect_tool_io(
    State(state): State<AppState>,
    Json(input): Json<ToolInspectionRequest>,
) -> Result<Json<ToolInspectionResponse>, ApiError> {
    let mut findings = Vec::new();
    for audit_id in input.audit_ids {
        for mapping in load_mappings_for_audit(&state, audit_id)? {
            if input.input.contains(&mapping.original_value) {
                findings.push(ToolLeakFinding {
                    audit_id,
                    location: "input".to_string(),
                    field_name: mapping.field_name.clone(),
                    field_type: mapping.field_type.clone(),
                    token: mapping.token.clone(),
                });
            }
            if input.output.contains(&mapping.original_value) {
                findings.push(ToolLeakFinding {
                    audit_id,
                    location: "output".to_string(),
                    field_name: mapping.field_name,
                    field_type: mapping.field_type,
                    token: mapping.token,
                });
            }
        }
    }

    Ok(Json(ToolInspectionResponse {
        tool_name: input.tool_name,
        passed: findings.is_empty(),
        unauthorized_sensitive_count: findings.len(),
        findings,
    }))
}

async fn session_risk(Json(input): Json<SessionRiskRequest>) -> Json<SessionRiskResponse> {
    let exposure_events = input
        .events
        .iter()
        .filter(|event| {
            event.external_view_digest.is_some()
                || event.privacy_budget_epsilon.unwrap_or(0.0) > 0.0
        })
        .count();
    let epsilon_total = input
        .events
        .iter()
        .map(|event| event.privacy_budget_epsilon.unwrap_or(0.0))
        .sum::<f64>();
    let risk_score = exposure_events as f64 + epsilon_total;
    Json(SessionRiskResponse {
        session_id: input.session_id,
        event_count: input.events.len(),
        exposure_events,
        epsilon_total,
        risk_score,
        passed: risk_score <= input.risk_bound,
        risk_bound: input.risk_bound,
    })
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

fn load_state() -> Result<AppState> {
    let policy_path = env::var("PRIVAGATE_POLICY_PATH")
        .context("PRIVAGATE_POLICY_PATH must point to a local policy JSON file")?;
    let hmac_key = env::var("PRIVAGATE_HMAC_KEY")
        .context("PRIVAGATE_HMAC_KEY must be set from local secret storage")?;
    let mapping_log_path = env::var("PRIVAGATE_MAPPING_LOG")
        .unwrap_or_else(|_| "data/local-mappings.jsonl".to_string());
    let audit_postgres_url = env::var("PRIVAGATE_AUDIT_POSTGRES_URL").ok();
    let audit_sink = match env::var("PRIVAGATE_AUDIT_POSTGRES_URL") {
        Ok(connection_string) if !connection_string.trim().is_empty() => {
            AuditSink::Postgres { connection_string }
        }
        _ => AuditSink::Jsonl {
            path: PathBuf::from(
                env::var("PRIVAGATE_AUDIT_LOG").unwrap_or_else(|_| "data/audit.jsonl".to_string()),
            ),
        },
    };
    let policy_json = fs::read_to_string(&policy_path)
        .with_context(|| format!("failed to read policy file: {policy_path}"))?;
    let policy: Policy = serde_json::from_str(&policy_json)
        .with_context(|| format!("failed to parse policy file: {policy_path}"))?;
    let review_mode = parse_review_mode(
        &env::var("PRIVAGATE_REVIEW_MODE").unwrap_or_else(|_| "off".to_string()),
    )?;
    let review_store = load_review_store(audit_postgres_url.as_deref());
    let model_adapter = load_model_adapter()?;

    Ok(AppState {
        policy: Arc::new(policy),
        hmac_key: Arc::new(hmac_key.into_bytes()),
        mapping_log: Arc::new(Mutex::new(MappingLog {
            path: PathBuf::from(mapping_log_path),
        })),
        audit_sink: Arc::new(Mutex::new(audit_sink)),
        model_adapter,
        review_mode,
        review_store: Arc::new(review_store),
    })
}

fn load_review_store(audit_postgres_url: Option<&str>) -> ReviewStore {
    if let Ok(connection_string) = env::var("PRIVAGATE_REVIEW_POSTGRES_URL") {
        if !connection_string.trim().is_empty() {
            return ReviewStore::Postgres { connection_string };
        }
    }

    if let Some(connection_string) = audit_postgres_url
        .map(str::trim)
        .filter(|connection_string| !connection_string.is_empty())
    {
        return ReviewStore::Postgres {
            connection_string: connection_string.to_string(),
        };
    }

    ReviewStore::Jsonl {
        path: PathBuf::from(
            env::var("PRIVAGATE_REVIEW_LOG")
                .unwrap_or_else(|_| "data/manual-review.jsonl".to_string()),
        ),
    }
}

fn load_model_adapter() -> Result<Arc<dyn ModelAdapter>> {
    let adapter_name = env::var("PRIVAGATE_MODEL_ADAPTER")
        .unwrap_or_else(|_| "disabled".to_string())
        .trim()
        .to_ascii_lowercase();

    let adapter: Arc<dyn ModelAdapter> = match adapter_name.as_str() {
        "" | "disabled" | "off" | "none" => Arc::new(DisabledModelAdapter),
        "dry_run" | "dry-run" | "dryrun" => Arc::new(DryRunAdapter),
        other => {
            return Err(anyhow::anyhow!(
                "invalid PRIVAGATE_MODEL_ADAPTER={other}; use disabled or dry_run"
            ))
        }
    };

    Ok(adapter)
}

async fn healthz() -> Json<Health> {
    Json(Health {
        status: "ok",
        service: SERVICE_NAME,
    })
}

async fn project(
    State(state): State<AppState>,
    Json(input): Json<ProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let effective_policy = project_policy(&state.policy, input.task_profile.as_deref());
    let mut output = process_document(input.input, &effective_policy, &state.hmac_key)?;
    let task_contract_assessment =
        assess_task_contract(&state.policy, &output.utility_report.task_profile);
    let manual_review =
        apply_projection_dispatch_controls(&state, &mut output, &task_contract_assessment).await?;
    persist_local_mappings(&state, &output)?;
    persist_audit_summary(&state, &output, Some(&task_contract_assessment)).await?;
    persist_projection_control_events(
        &state,
        &output,
        &task_contract_assessment,
        manual_review.as_ref(),
    )
    .await?;
    Ok(Json(ProjectResponse {
        output,
        manual_review,
        task_contract_assessment,
    }))
}

async fn statistics(
    State(state): State<AppState>,
    Json(input): Json<GatewayInput>,
) -> Result<Json<StatisticOutput>, ApiError> {
    let output = compute_statistics(&input.payload, &state.policy.statistics, &state.hmac_key)?;
    persist_statistic_audit(&state, &output).await?;
    Ok(Json(output))
}

async fn inspect_output(
    State(state): State<AppState>,
    Json(input): Json<OutputInspectionRequest>,
) -> Result<Json<OutputInspectionResponse>, ApiError> {
    let findings = load_mappings_for_audit(&state, input.audit_id)?
        .into_iter()
        .filter(|mapping| input.output.contains(&mapping.original_value))
        .map(|mapping| OutputLeakFinding {
            field_name: mapping.field_name,
            field_type: mapping.field_type,
            token: mapping.token,
        })
        .collect::<Vec<_>>();

    Ok(Json(OutputInspectionResponse {
        audit_id: input.audit_id,
        passed: findings.is_empty(),
        unauthorized_sensitive_output_count: findings.len(),
        findings,
    }))
}

async fn model_dispatch(
    State(state): State<AppState>,
    Json(input): Json<ModelDispatchRequest>,
) -> Result<Json<ModelDispatchResponse>, ApiError> {
    let external_view_digest = privagate_core::digest::sha256_json(&input.external_view)?;
    let adapter_capabilities = state.model_adapter.capabilities();
    let task_contract_assessment = assess_task_contract(&state.policy, &input.task_profile);
    if !task_contract_assessment.dispatch_allowed {
        persist_dispatch_policy_block_event(
            &state,
            input.audit_id,
            &external_view_digest,
            &input.provider,
            &adapter_capabilities,
            &task_contract_assessment,
        )
        .await?;
        return Ok(Json(ModelDispatchResponse {
            provider: input.provider.clone(),
            dispatched: false,
            status: format!(
                "task contract blocked dispatch: {}",
                task_contract_assessment.issues.join("; ")
            ),
            output: None,
            audit_id: input.audit_id,
            external_view_digest: Some(external_view_digest),
            blocked_by_review: false,
            blocked_by_policy: true,
            adapter_capabilities,
        }));
    }
    if let Some(reason) =
        adapter_capability_block_reason(&state.policy, &input.task_profile, &adapter_capabilities)
    {
        persist_adapter_capability_block_event(
            &state,
            input.audit_id,
            &external_view_digest,
            &input.provider,
            &adapter_capabilities,
            &input.task_profile,
            &reason,
        )
        .await?;
        return Ok(Json(ModelDispatchResponse {
            provider: input.provider.clone(),
            dispatched: false,
            status: format!("adapter capability blocked dispatch: {reason}"),
            output: None,
            audit_id: input.audit_id,
            external_view_digest: Some(external_view_digest),
            blocked_by_review: false,
            blocked_by_policy: true,
            adapter_capabilities,
        }));
    }
    if let Some(reason) =
        review_dispatch_block_reason(&state, input.audit_id, &external_view_digest).await?
    {
        return Ok(Json(ModelDispatchResponse {
            provider: input.provider,
            dispatched: false,
            status: format!("manual review gate blocked dispatch: {reason}"),
            output: None,
            audit_id: input.audit_id,
            external_view_digest: Some(external_view_digest),
            blocked_by_review: true,
            blocked_by_policy: false,
            adapter_capabilities,
        }));
    }

    let mut response = state.model_adapter.dispatch(input);
    response.external_view_digest = Some(external_view_digest);
    Ok(Json(response))
}

async fn validate_route_plan(
    State(state): State<AppState>,
    Json(input): Json<RoutePlanValidationRequest>,
) -> Result<Json<RoutePlanValidationResponse>, ApiError> {
    let response = assess_route_plan(&state, &input).await?;
    persist_route_plan_validation_event(&state, &response).await?;
    Ok(Json(response))
}

async fn execute_route_plan(
    State(state): State<AppState>,
    Json(input): Json<RoutePlanExecutionRequest>,
) -> Result<Json<RoutePlanExecutionResponse>, ApiError> {
    let response = run_route_plan_execution(&state, &input).await?;
    persist_route_plan_execution_event(&state, &response).await?;
    Ok(Json(response))
}

async fn validate_shard_plan(
    State(state): State<AppState>,
    Json(input): Json<ShardPlanValidationRequest>,
) -> Result<Json<ShardPlanValidationResponse>, ApiError> {
    let route_plan = assess_route_plan(&state, &input.route_plan).await?;
    let local_aggregation_summary = build_local_aggregation_summary_for_validation(
        &state.policy,
        state.review_mode,
        &route_plan,
        &input.aggregation_rules,
    )?;
    let response = ShardPlanValidationResponse {
        route_plan,
        aggregation_rules: input.aggregation_rules,
        local_aggregation_summary,
    };
    persist_shard_plan_validation_event(&state, &response).await?;
    Ok(Json(response))
}

async fn execute_shard_plan(
    State(state): State<AppState>,
    Json(input): Json<ShardPlanExecutionRequest>,
) -> Result<Json<ShardPlanExecutionResponse>, ApiError> {
    let route_plan_execution =
        run_route_plan_execution(&state, &input.route_plan_execution).await?;
    let local_aggregation_summary = build_local_aggregation_summary_for_execution(
        &state.policy,
        state.review_mode,
        &route_plan_execution,
        &input.aggregation_rules,
    )?;
    let response = ShardPlanExecutionResponse {
        route_plan_execution,
        aggregation_rules: input.aggregation_rules,
        local_aggregation_summary,
    };
    persist_shard_plan_execution_event(&state, &response).await?;
    Ok(Json(response))
}

async fn bind_shard_promotion(
    State(state): State<AppState>,
    Json(input): Json<ShardPromotionBindingRequest>,
) -> Result<Json<ShardPromotionBindingResponse>, ApiError> {
    let response = bind_local_shard_promotion(&state, &input).await?;
    persist_shard_promotion_binding_event(&state, &response).await?;
    if let Some(review) = response.manual_review.as_ref() {
        persist_review_event(&state, "pending", review).await?;
    }
    Ok(Json(response))
}

async fn bind_local_shard_promotion(
    state: &AppState,
    input: &ShardPromotionBindingRequest,
) -> Result<ShardPromotionBindingResponse> {
    let group_id = input.group_id.trim();
    if group_id.is_empty() {
        return Err(anyhow::anyhow!("group_id must not be empty"));
    }

    let replay_issues = execution_replay_issues(&input.route_plan_execution);
    let local_aggregation_summary = build_local_aggregation_summary_for_execution(
        &state.policy,
        ReviewMode::Off,
        &input.route_plan_execution,
        &input.aggregation_rules,
    )?;

    let Some(group) = local_aggregation_summary
        .groups
        .iter()
        .find(|group| group.group_id == group_id)
    else {
        return Ok(ShardPromotionBindingResponse {
            route_id: input.route_plan_execution.route_id.clone(),
            group_id: group_id.to_string(),
            binding_created: false,
            ready_for_follow_up_route: false,
            issues: vec![format!(
                "group_id={} was not found in local aggregation summary",
                group_id
            )],
            source_local_aggregation_digest: None,
            source_local_only_output_digest: None,
            promotion: None,
            follow_up_binding: None,
            manual_review: None,
        });
    };

    let Some(promotion) = group.promotion.clone() else {
        return Ok(ShardPromotionBindingResponse {
            route_id: input.route_plan_execution.route_id.clone(),
            group_id: group_id.to_string(),
            binding_created: false,
            ready_for_follow_up_route: false,
            issues: vec![format!(
                "group_id={} has no promotion candidate because promotion rules are disabled",
                group_id
            )],
            source_local_aggregation_digest: group.local_aggregation_digest.clone(),
            source_local_only_output_digest: group.local_only_output_digest.clone(),
            promotion: None,
            follow_up_binding: None,
            manual_review: None,
        });
    };

    let mut issues = replay_issues;
    if !promotion.promotion_allowed {
        issues.extend(promotion.issues.iter().cloned());
    }

    let (binding_created, ready_for_follow_up_route, follow_up_binding, manual_review) = if issues
        .is_empty()
    {
        if let (
            Some(task_profile),
            Some(external_view),
            Some(source_input_digest),
            Some(utility_assessment),
        ) = (
            promotion.candidate_task_profile.clone(),
            promotion.external_view_candidate.clone(),
            group.local_only_output_digest.clone(),
            promotion.utility_assessment.clone(),
        ) {
            let audit_id = Uuid::new_v4();
            let external_view_digest = privagate_core::digest::sha256_json(&external_view)?;
            let manual_review = create_manual_review_for_digest_if_required(
                state,
                audit_id,
                &external_view_digest,
                "manual review required before follow-up dispatch of promoted aggregation candidate",
            )
                .await?;
            let blocked = manual_review.is_some();
            let follow_up_binding = FollowUpViewBinding {
                task_profile,
                audit_summary: privagate_core::report::AuditSummary {
                    audit_id,
                    input_digest: source_input_digest,
                    external_view_digest,
                    policy_version: state.policy.policy_version.clone(),
                    blocked,
                },
                external_view,
                utility_assessment,
            };
            (true, !blocked, Some(follow_up_binding), manual_review)
        } else {
            issues.push(
                "promotion candidate is missing task_profile, external_view, utility_assessment, or source digest"
                    .to_string(),
            );
            (false, false, None, None)
        }
    } else {
        (false, false, None, None)
    };

    Ok(ShardPromotionBindingResponse {
        route_id: input.route_plan_execution.route_id.clone(),
        group_id: group_id.to_string(),
        binding_created,
        ready_for_follow_up_route,
        issues,
        source_local_aggregation_digest: group.local_aggregation_digest.clone(),
        source_local_only_output_digest: group.local_only_output_digest.clone(),
        promotion: Some(promotion),
        follow_up_binding,
        manual_review,
    })
}

async fn run_route_plan_execution(
    state: &AppState,
    input: &RoutePlanExecutionRequest,
) -> Result<RoutePlanExecutionResponse> {
    let validation = assess_route_plan(&state, &input.route_plan).await?;
    let runtime_adapter_capabilities = state.model_adapter.capabilities();
    let mut stages = Vec::with_capacity(input.route_plan.stages.len());
    let mut halted = false;

    for (stage_request, validation_stage) in input
        .route_plan
        .stages
        .iter()
        .cloned()
        .zip(validation.stages.iter().cloned())
    {
        if halted {
            stages.push(RouteStageExecution {
                validation: validation_stage,
                executed: false,
                runtime_issues: Vec::new(),
                skipped_reason: Some(
                    "route execution stopped after an earlier stage was blocked or not dispatched"
                        .to_string(),
                ),
                dispatch_output_digest: None,
                dispatch_response: None,
            });
            continue;
        }

        let runtime_issues = route_stage_runtime_issues(
            &state.policy,
            &stage_request,
            &runtime_adapter_capabilities,
        );
        if !validation_stage.dispatch_allowed || !runtime_issues.is_empty() {
            if input.stop_on_block {
                halted = true;
            }
            stages.push(RouteStageExecution {
                validation: validation_stage,
                executed: false,
                runtime_issues,
                skipped_reason: None,
                dispatch_output_digest: None,
                dispatch_response: None,
            });
            continue;
        }

        let mut dispatch_response = state.model_adapter.dispatch(ModelDispatchRequest {
            provider: stage_request.provider,
            task_profile: stage_request.task_profile,
            audit_id: stage_request.audit_id,
            external_view: stage_request.external_view,
        });
        dispatch_response.external_view_digest =
            Some(validation_stage.external_view_digest.clone());
        let stage_dispatched = dispatch_response.dispatched
            && !dispatch_response.blocked_by_policy
            && !dispatch_response.blocked_by_review;
        if input.stop_on_block && !stage_dispatched {
            halted = true;
        }
        let dispatch_output_digest = dispatch_response
            .output
            .as_deref()
            .map(|output| privagate_core::digest::sha256_bytes(output.as_bytes()));

        stages.push(RouteStageExecution {
            validation: validation_stage,
            executed: true,
            runtime_issues,
            skipped_reason: None,
            dispatch_output_digest,
            dispatch_response: Some(dispatch_response),
        });
    }

    let executed_stage_count = stages.iter().filter(|stage| stage.executed).count();
    let dispatched_stage_count = stages
        .iter()
        .filter(|stage| {
            stage
                .dispatch_response
                .as_ref()
                .is_some_and(|response| response.dispatched)
        })
        .count();
    let all_stages_dispatched = dispatched_stage_count == stages.len();
    let response = RoutePlanExecutionResponse {
        route_id: validation.route_id,
        aggregation_strategy: validation.aggregation_strategy,
        residual_risk_notes: validation.residual_risk_notes,
        dispatch_allowed: validation.dispatch_allowed,
        stop_on_block: input.stop_on_block,
        halted,
        executed_stage_count,
        dispatched_stage_count,
        all_stages_dispatched,
        runtime_adapter_capabilities,
        stages,
    };
    Ok(response)
}

fn execution_replay_issues(route_plan_execution: &RoutePlanExecutionResponse) -> Vec<String> {
    let mut issues = Vec::new();
    for stage in &route_plan_execution.stages {
        match (&stage.dispatch_response, &stage.dispatch_output_digest) {
            (Some(response), Some(expected_digest)) => match response.output.as_deref() {
                Some(output) => {
                    let actual_digest = privagate_core::digest::sha256_bytes(output.as_bytes());
                    if &actual_digest != expected_digest {
                        issues.push(format!(
                            "stage_id={} dispatch_output_digest mismatch: expected {} but recomputed {}",
                            stage.validation.stage_id, expected_digest, actual_digest
                        ));
                    }
                }
                None => issues.push(format!(
                    "stage_id={} includes dispatch_output_digest but dispatch response has no output",
                    stage.validation.stage_id
                )),
            },
            (Some(response), None) if response.output.is_some() => issues.push(format!(
                "stage_id={} includes dispatch output but no dispatch_output_digest",
                stage.validation.stage_id
            )),
            (None, Some(_)) => issues.push(format!(
                "stage_id={} includes dispatch_output_digest but no dispatch_response",
                stage.validation.stage_id
            )),
            _ => {}
        }

        if stage.executed && stage.dispatch_response.is_none() {
            issues.push(format!(
                "stage_id={} is marked executed but has no dispatch_response",
                stage.validation.stage_id
            ));
        }
        if !stage.executed && stage.dispatch_response.is_some() {
            issues.push(format!(
                "stage_id={} has dispatch_response even though executed=false",
                stage.validation.stage_id
            ));
        }
    }
    issues
}

async fn assess_route_plan(
    state: &AppState,
    input: &RoutePlanValidationRequest,
) -> Result<RoutePlanValidationResponse> {
    if input.stages.is_empty() {
        return Err(anyhow::anyhow!(
            "route plan must contain at least one stage"
        ));
    }

    let route_id = input
        .route_id
        .as_deref()
        .map(str::trim)
        .filter(|route_id| !route_id.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut stages = Vec::with_capacity(input.stages.len());
    for stage in &input.stages {
        stages.push(assess_route_stage(state, stage).await?);
    }

    Ok(RoutePlanValidationResponse {
        route_id,
        aggregation_strategy: input.aggregation_strategy.clone(),
        residual_risk_notes: input.residual_risk_notes.clone(),
        dispatch_allowed: stages.iter().all(|stage| stage.dispatch_allowed),
        stages,
    })
}

async fn assess_route_stage(
    state: &AppState,
    stage: &RouteStageRequest,
) -> Result<RouteStageValidation> {
    let external_view_digest = privagate_core::digest::sha256_json(&stage.external_view)?;
    let task_contract_assessment = assess_task_contract(&state.policy, &stage.task_profile);
    let mut issues = task_contract_assessment.issues.clone();

    let adapter_issues = state
        .policy
        .adapter_contract_issues(&stage.task_profile, &stage.adapter_class);
    let blocked_by_policy =
        !task_contract_assessment.dispatch_allowed || !adapter_issues.is_empty();
    issues.extend(adapter_issues);

    let mut blocked_by_review = false;
    if !blocked_by_policy {
        if let Some(reason) =
            review_dispatch_block_reason(state, stage.audit_id, &external_view_digest).await?
        {
            blocked_by_review = true;
            issues.push(reason);
        }
    }

    Ok(RouteStageValidation {
        stage_id: stage.stage_id.clone(),
        provider: stage.provider.clone(),
        task_profile: stage.task_profile.clone(),
        adapter_class: stage.adapter_class.clone(),
        audit_id: stage.audit_id,
        shard_group: stage.shard_group.clone(),
        shard_id: stage.shard_id.clone(),
        external_view_digest,
        blocked_by_policy,
        blocked_by_review,
        dispatch_allowed: !blocked_by_policy && !blocked_by_review,
        issues,
        task_contract_assessment,
    })
}

fn route_stage_runtime_issues(
    policy: &Policy,
    stage: &RouteStageRequest,
    runtime_adapter_capabilities: &AdapterCapabilities,
) -> Vec<String> {
    let mut issues = Vec::new();
    if stage.adapter_class != runtime_adapter_capabilities.adapter_class {
        issues.push(format!(
            "route stage declared adapter_class={} but configured adapter_class={}",
            stage.adapter_class.as_str(),
            runtime_adapter_capabilities.adapter_class.as_str(),
        ));
    }
    if let Some(reason) =
        adapter_capability_block_reason(policy, &stage.task_profile, runtime_adapter_capabilities)
    {
        issues.push(format!("configured adapter blocked dispatch: {reason}"));
    }
    issues
}

fn build_local_aggregation_summary_for_validation(
    policy: &Policy,
    review_mode: ReviewMode,
    route_plan: &RoutePlanValidationResponse,
    aggregation_rules: &ShardAggregationRules,
) -> Result<LocalAggregationSummary> {
    let stages = route_plan
        .stages
        .iter()
        .map(|stage| NormalizedShardStage {
            stage_id: stage.stage_id.clone(),
            provider: stage.provider.clone(),
            shard_group: stage.shard_group.clone(),
            shard_id: stage.shard_id.clone(),
            external_view_digest: stage.external_view_digest.clone(),
            dispatch_allowed: stage.dispatch_allowed,
            executed: false,
            dispatched: false,
            dispatch_output_digest: None,
            dispatch_output: None,
        })
        .collect::<Vec<_>>();
    build_local_aggregation_summary(
        policy,
        review_mode,
        route_plan.aggregation_strategy.as_deref(),
        &stages,
        aggregation_rules,
        false,
    )
}

fn build_local_aggregation_summary_for_execution(
    policy: &Policy,
    review_mode: ReviewMode,
    route_plan: &RoutePlanExecutionResponse,
    aggregation_rules: &ShardAggregationRules,
) -> Result<LocalAggregationSummary> {
    let stages = route_plan
        .stages
        .iter()
        .map(|stage| NormalizedShardStage {
            stage_id: stage.validation.stage_id.clone(),
            provider: stage.validation.provider.clone(),
            shard_group: stage.validation.shard_group.clone(),
            shard_id: stage.validation.shard_id.clone(),
            external_view_digest: stage.validation.external_view_digest.clone(),
            dispatch_allowed: stage.validation.dispatch_allowed,
            executed: stage.executed,
            dispatched: stage
                .dispatch_response
                .as_ref()
                .is_some_and(|response| response.dispatched),
            dispatch_output_digest: stage.dispatch_output_digest.clone(),
            dispatch_output: stage
                .dispatch_response
                .as_ref()
                .and_then(|response| response.output.clone()),
        })
        .collect::<Vec<_>>();
    build_local_aggregation_summary(
        policy,
        review_mode,
        route_plan.aggregation_strategy.as_deref(),
        &stages,
        aggregation_rules,
        true,
    )
}

fn build_local_aggregation_summary(
    policy: &Policy,
    review_mode: ReviewMode,
    aggregation_strategy: Option<&str>,
    stages: &[NormalizedShardStage],
    aggregation_rules: &ShardAggregationRules,
    require_execution_completion: bool,
) -> Result<LocalAggregationSummary> {
    let mut issues = Vec::new();
    let mut grouped_stages: BTreeMap<String, Vec<NormalizedShardStage>> = BTreeMap::new();

    for stage in stages {
        if aggregation_rules.require_shard_metadata {
            if stage.shard_group.as_deref().is_none_or(str::is_empty) {
                issues.push(format!("stage_id={} missing shard_group", stage.stage_id));
            }
            if stage.shard_id.as_deref().is_none_or(str::is_empty) {
                issues.push(format!("stage_id={} missing shard_id", stage.stage_id));
            }
        }

        let group_id = stage
            .shard_group
            .as_deref()
            .map(str::trim)
            .filter(|group_id| !group_id.is_empty())
            .unwrap_or("ungrouped")
            .to_string();
        grouped_stages
            .entry(group_id)
            .or_default()
            .push(stage.clone());
    }

    for expected_group in &aggregation_rules.expected_groups {
        if !grouped_stages.contains_key(&expected_group.group_id) {
            issues.push(format!(
                "expected shard_group={} is missing from the plan",
                expected_group.group_id
            ));
        }
    }

    let mut groups = Vec::new();
    for (group_id, group_stages) in grouped_stages {
        let group_strategy = strategy_for_group(aggregation_rules, &group_id);
        let promotion_rules = promotion_rules_for_group(aggregation_rules, &group_id);
        let mut group_issues = Vec::new();
        let expected_min_shards = aggregation_rules
            .expected_groups
            .iter()
            .find(|group| group.group_id == group_id)
            .map(|group| group.min_shards);
        if let Some(min_shards) = expected_min_shards {
            if group_stages.len() < min_shards {
                group_issues.push(format!(
                    "shard_group={} expected at least {} shards but observed {}",
                    group_id,
                    min_shards,
                    group_stages.len()
                ));
            }
        }

        let unique_provider_count = group_stages
            .iter()
            .map(|stage| stage.provider.clone())
            .collect::<BTreeSet<_>>()
            .len();
        if aggregation_rules.require_distinct_providers
            && unique_provider_count < group_stages.len()
        {
            group_issues.push(format!(
                "shard_group={} requires distinct providers but observed {} providers across {} stages",
                group_id,
                unique_provider_count,
                group_stages.len()
            ));
        }

        let mut seen_shard_ids = BTreeSet::new();
        let mut duplicate_shard_ids = BTreeSet::new();
        for shard_id in group_stages
            .iter()
            .filter_map(|stage| stage.shard_id.clone())
        {
            if !seen_shard_ids.insert(shard_id.clone()) {
                duplicate_shard_ids.insert(shard_id);
            }
        }
        if !duplicate_shard_ids.is_empty() {
            group_issues.push(format!(
                "shard_group={} contains duplicate shard_id values: {}",
                group_id,
                duplicate_shard_ids
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let dispatch_allowed_stage_count = group_stages
            .iter()
            .filter(|stage| stage.dispatch_allowed)
            .count();
        let executed_stage_count = group_stages.iter().filter(|stage| stage.executed).count();
        let dispatched_stage_count = group_stages.iter().filter(|stage| stage.dispatched).count();
        if require_execution_completion && dispatched_stage_count < group_stages.len() {
            group_issues.push(format!(
                "shard_group={} has only {}/{} dispatched stages",
                group_id,
                dispatched_stage_count,
                group_stages.len()
            ));
        }

        let stage_ids = group_stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<Vec<_>>();
        let shard_ids = group_stages
            .iter()
            .filter_map(|stage| stage.shard_id.clone())
            .collect::<Vec<_>>();
        let output_digests = group_stages
            .iter()
            .filter_map(|stage| stage.dispatch_output_digest.clone())
            .collect::<Vec<_>>();
        let all_stage_outputs_bound =
            dispatched_stage_count > 0 && output_digests.len() == dispatched_stage_count;
        let local_only_output = if require_execution_completion
            && dispatched_stage_count == group_stages.len()
            && !group_stages.is_empty()
        {
            let outcome = build_local_only_aggregation_output(&group_strategy, &group_stages);
            for issue in outcome.issues {
                group_issues.push(format!("shard_group={group_id} {issue}"));
            }
            outcome.output
        } else {
            None
        };
        let local_aggregation_digest =
            if dispatched_stage_count == group_stages.len() && !group_stages.is_empty() {
                Some(privagate_core::digest::sha256_json(&serde_json::json!({
                    "aggregation_strategy": aggregation_strategy,
                    "group_strategy": &group_strategy,
                    "group_id": &group_id,
                    "stage_bindings": group_stages.iter().map(|stage| serde_json::json!({
                        "stage_id": &stage.stage_id,
                        "shard_id": &stage.shard_id,
                        "external_view_digest": &stage.external_view_digest,
                        "dispatch_output_digest": &stage.dispatch_output_digest,
                    })).collect::<Vec<_>>(),
                }))?)
            } else {
                None
            };
        let local_only_output_digest = local_only_output
            .as_ref()
            .map(privagate_core::digest::sha256_json)
            .transpose()?;
        let promotion = build_local_promotion_summary(
            policy,
            review_mode,
            require_execution_completion,
            &promotion_rules,
            &group_issues,
            local_only_output.as_ref(),
        )?;

        groups.push(LocalAggregationGroupSummary {
            group_id,
            aggregation_strategy: group_strategy,
            expected_min_shards,
            observed_stage_count: group_stages.len(),
            unique_provider_count,
            dispatch_allowed_stage_count,
            executed_stage_count,
            dispatched_stage_count,
            all_stage_outputs_bound,
            local_aggregation_digest,
            local_only_output_digest,
            local_only_output,
            promotion,
            stage_ids,
            shard_ids,
            output_digests,
        });
        issues.extend(group_issues);
    }

    let ready_for_local_aggregation = issues.is_empty()
        && stages.iter().all(|stage| stage.dispatch_allowed)
        && (!require_execution_completion || stages.iter().all(|stage| stage.dispatched));

    Ok(LocalAggregationSummary {
        aggregation_strategy: aggregation_strategy.map(str::to_string),
        ready_for_local_aggregation,
        issue_count: issues.len(),
        issues,
        groups,
    })
}

fn strategy_for_group(
    aggregation_rules: &ShardAggregationRules,
    group_id: &str,
) -> LocalAggregationStrategy {
    aggregation_rules
        .expected_groups
        .iter()
        .find(|group| group.group_id == group_id)
        .and_then(|group| group.strategy.clone())
        .unwrap_or_else(|| aggregation_rules.strategy.clone())
}

fn promotion_rules_for_group(
    aggregation_rules: &ShardAggregationRules,
    group_id: &str,
) -> AggregationPromotionRules {
    aggregation_rules
        .expected_groups
        .iter()
        .find(|group| group.group_id == group_id)
        .and_then(|group| group.promotion.clone())
        .unwrap_or_else(|| aggregation_rules.promotion.clone())
}

fn build_local_promotion_summary(
    policy: &Policy,
    review_mode: ReviewMode,
    require_execution_completion: bool,
    promotion_rules: &AggregationPromotionRules,
    group_issues: &[String],
    local_only_output: Option<&serde_json::Value>,
) -> Result<Option<LocalPromotionSummary>> {
    if promotion_rules.mode == AggregationPromotionMode::LocalOnly {
        return Ok(None);
    }

    let mut issues = Vec::new();
    if !require_execution_completion {
        issues.push("promotion assessment requires executed shard outputs".to_string());
    }
    issues.extend(group_issues.iter().cloned());

    let candidate_task_profile = promotion_rules
        .candidate_task_profile
        .as_deref()
        .map(str::trim)
        .filter(|task_profile| !task_profile.is_empty())
        .map(str::to_string);

    let (candidate_content_type, task_contract_assessment, utility_assessment, blocking_issues) =
        if let Some(output) = local_only_output {
            let candidate_content_type = infer_candidate_content_type(output);
            let task_contract_assessment = candidate_task_profile
                .as_deref()
                .map(|task_profile| assess_task_contract(policy, task_profile));
            let mut blocking_issues =
                promotion_constraint_issues(output, &candidate_content_type, promotion_rules)?;
            let utility_assessment = match &candidate_task_profile {
                Some(task_profile) => {
                    if let Some(assessment) = task_contract_assessment.as_ref() {
                        if !assessment.dispatch_allowed {
                            blocking_issues.extend(assessment.issues.iter().cloned());
                            None
                        } else {
                            Some(evaluate_follow_up_utility(
                                policy,
                                task_profile,
                                output,
                                &promotion_rules.utility_verification,
                            ))
                        }
                    } else {
                        Some(evaluate_follow_up_utility(
                            policy,
                            task_profile,
                            output,
                            &promotion_rules.utility_verification,
                        ))
                    }
                }
                None => {
                    blocking_issues.push("promotion requires candidate_task_profile".to_string());
                    None
                }
            };

            if let Some(assessment) = utility_assessment.as_ref() {
                if !assessment.verification_passed {
                    blocking_issues.extend(assessment.issues.iter().cloned());
                }
            }

            (
                Some(candidate_content_type),
                task_contract_assessment,
                utility_assessment,
                blocking_issues,
            )
        } else {
            (
                None,
                None,
                None,
                vec!["local aggregation did not materialize a promotable output".to_string()],
            )
        };

    issues.extend(blocking_issues.iter().cloned());
    let promotion_allowed = issues.is_empty();
    let external_view_candidate = if promotion_allowed {
        local_only_output.map(|output| privagate_core::ExternalView {
            content_type: candidate_content_type
                .clone()
                .expect("candidate content type should exist when promotion succeeds"),
            payload: output.clone(),
        })
    } else {
        None
    };
    let external_view_candidate_digest = external_view_candidate
        .as_ref()
        .map(privagate_core::digest::sha256_json)
        .transpose()?;

    let mut ready_for_follow_up_route = promotion_allowed;
    if promotion_allowed && review_mode == ReviewMode::Manual {
        ready_for_follow_up_route = false;
        issues.push(
            "manual review mode requires a projection-backed audit_id before follow-up dispatch"
                .to_string(),
        );
    }

    Ok(Some(LocalPromotionSummary {
        mode: promotion_rules.mode.clone(),
        promotion_allowed,
        ready_for_follow_up_route,
        candidate_task_profile,
        candidate_content_type,
        task_contract_assessment,
        utility_assessment,
        issues,
        external_view_candidate_digest,
        external_view_candidate,
    }))
}

fn infer_candidate_content_type(
    output: &serde_json::Value,
) -> privagate_core::transform::ContentType {
    match output {
        serde_json::Value::String(_) => privagate_core::transform::ContentType::Text,
        _ => privagate_core::transform::ContentType::Json,
    }
}

fn promotion_constraint_issues(
    output: &serde_json::Value,
    candidate_content_type: &privagate_core::transform::ContentType,
    promotion_rules: &AggregationPromotionRules,
) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    if !promotion_rules.allowed_content_types.is_empty()
        && !promotion_rules
            .allowed_content_types
            .contains(candidate_content_type)
    {
        issues.push(format!(
            "promotion candidate content_type={} is not allowed by policy",
            content_type_label(candidate_content_type)
        ));
    }

    if let Some(max_serialized_bytes) = promotion_rules.max_serialized_bytes {
        let size = serde_json::to_vec(output)?.len();
        if size > max_serialized_bytes {
            issues.push(format!(
                "promotion candidate serialized size {} bytes exceeds max_serialized_bytes={}",
                size, max_serialized_bytes
            ));
        }
    }

    if let Some(max_text_chars) = promotion_rules.max_text_chars {
        if let Some(text) = output.as_str() {
            let char_count = text.chars().count();
            if char_count > max_text_chars {
                issues.push(format!(
                    "promotion candidate text length {} exceeds max_text_chars={}",
                    char_count, max_text_chars
                ));
            }
        }
    }

    if let Some(max_array_items) = promotion_rules.max_array_items {
        if let Some(items) = output.as_array() {
            if items.len() > max_array_items {
                issues.push(format!(
                    "promotion candidate array length {} exceeds max_array_items={}",
                    items.len(),
                    max_array_items
                ));
            }
        }
    }

    if let Some(max_object_keys) = promotion_rules.max_object_keys {
        if let Some(object) = output.as_object() {
            if object.len() > max_object_keys {
                issues.push(format!(
                    "promotion candidate object key count {} exceeds max_object_keys={}",
                    object.len(),
                    max_object_keys
                ));
            }
        }
    }

    Ok(issues)
}

fn content_type_label(content_type: &privagate_core::transform::ContentType) -> &'static str {
    match content_type {
        privagate_core::transform::ContentType::Text => "text",
        privagate_core::transform::ContentType::Json => "json",
        privagate_core::transform::ContentType::CsvRows => "csv_rows",
    }
}

fn evaluate_follow_up_utility(
    policy: &Policy,
    task_profile: &str,
    payload: &serde_json::Value,
    rules: &PromotionUtilityVerificationRules,
) -> FollowUpUtilityAssessment {
    let task_contract = policy.task_contracts.get(task_profile);
    let task_profile_utility = task_contract
        .map(|contract| contract.promotion_utility.clone())
        .unwrap_or_default();
    let required_fields = task_contract
        .map(|contract| contract.required_fields.clone())
        .unwrap_or_default();
    let require_required_fields =
        rules.require_required_fields || task_profile_utility.require_required_fields;
    let missing_required_fields = if require_required_fields {
        required_fields
            .iter()
            .filter(|field_name| !payload_contains_field(payload, field_name))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let preserved_required_field_count = required_fields
        .len()
        .saturating_sub(missing_required_fields.len());
    let required_field_presence =
        privagate_core::verify::preservation(required_fields.len(), preserved_required_field_count);

    let mut constraint_checks = task_profile_utility.required_constraint_checks;
    if rules.verify_constraint_results {
        constraint_checks.extend(enabled_constraint_checks_from_policy(policy));
    }
    constraint_checks.sort();
    constraint_checks.dedup();

    let constraint_results = if constraint_checks.is_empty() {
        Vec::new()
    } else {
        follow_up_constraint_results(payload, policy, &constraint_checks)
    };

    let mut issues = missing_required_fields
        .iter()
        .map(|field_name| {
            format!(
                "follow-up utility check missing required field={field_name} for task_profile={task_profile}"
            )
        })
        .collect::<Vec<_>>();
    for result in &constraint_results {
        if !result.passed {
            issues.push(format!(
                "follow-up utility check failed {}: {}",
                result.check, result.details
            ));
        }
    }

    FollowUpUtilityAssessment {
        task_profile: task_profile.to_string(),
        verification_passed: issues.is_empty(),
        required_field_presence,
        missing_required_fields,
        constraint_results,
        issues,
    }
}

fn enabled_constraint_checks_from_policy(
    policy: &Policy,
) -> Vec<privagate_core::UtilityConstraintCheck> {
    let mut checks = Vec::new();
    if policy.constraints.preserve_foreign_keys {
        checks.push(privagate_core::UtilityConstraintCheck::ForeignKeyValidity);
    }
    if policy.constraints.preserve_time_order {
        checks.push(privagate_core::UtilityConstraintCheck::TimeOrderValidity);
    }
    if policy.constraints.preserve_relations {
        checks.push(privagate_core::UtilityConstraintCheck::RelationPreservation);
    }
    checks
}

fn follow_up_constraint_results(
    payload: &serde_json::Value,
    policy: &Policy,
    checks: &[privagate_core::UtilityConstraintCheck],
) -> Vec<privagate_core::report::VerificationResult> {
    let mut results = Vec::new();

    for check in checks {
        match check {
            privagate_core::UtilityConstraintCheck::ForeignKeyValidity => {
                if !policy.constraints.preserve_foreign_keys {
                    results.push(privagate_core::report::VerificationResult {
                        check: "foreign_key_validity".to_string(),
                        passed: false,
                        details: "task_profile requires foreign_key_validity but policy does not enable preserve_foreign_keys".to_string(),
                    });
                    continue;
                }
                if policy.constraints.foreign_keys.is_empty() {
                    results.push(privagate_core::report::VerificationResult {
                        check: "foreign_key_validity".to_string(),
                        passed: false,
                        details: "task_profile requires foreign_key_validity but no foreign_keys are declared".to_string(),
                    });
                    continue;
                }
                for constraint in &policy.constraints.foreign_keys {
                    results.push(privagate_core::verify::foreign_key_result(
                        payload, constraint,
                    ));
                }
            }
            privagate_core::UtilityConstraintCheck::TimeOrderValidity => {
                if !policy.constraints.preserve_time_order {
                    results.push(privagate_core::report::VerificationResult {
                        check: "time_order_validity".to_string(),
                        passed: false,
                        details: "task_profile requires time_order_validity but policy does not enable preserve_time_order".to_string(),
                    });
                    continue;
                }
                if policy.constraints.time_orders.is_empty() {
                    results.push(privagate_core::report::VerificationResult {
                        check: "time_order_validity".to_string(),
                        passed: false,
                        details: "task_profile requires time_order_validity but no time_orders are declared".to_string(),
                    });
                    continue;
                }
                for constraint in &policy.constraints.time_orders {
                    results.push(privagate_core::verify::time_order_result(
                        payload, constraint,
                    ));
                }
            }
            privagate_core::UtilityConstraintCheck::RelationPreservation => {
                if !policy.constraints.preserve_relations {
                    results.push(privagate_core::report::VerificationResult {
                        check: "relation_preservation".to_string(),
                        passed: false,
                        details: "task_profile requires relation_preservation but policy does not enable preserve_relations".to_string(),
                    });
                    continue;
                }
                if policy.constraints.relations.is_empty() {
                    results.push(privagate_core::report::VerificationResult {
                        check: "relation_preservation".to_string(),
                        passed: false,
                        details: "task_profile requires relation_preservation but no relations are declared".to_string(),
                    });
                    continue;
                }
                for constraint in &policy.constraints.relations {
                    results.push(privagate_core::verify::relation_result(payload, constraint));
                }
            }
        }
    }

    results
}

fn payload_contains_field(payload: &serde_json::Value, field_name: &str) -> bool {
    match payload {
        serde_json::Value::Object(map) => {
            map.contains_key(field_name)
                || map
                    .values()
                    .any(|value| payload_contains_field(value, field_name))
        }
        serde_json::Value::Array(items) => items
            .iter()
            .any(|value| payload_contains_field(value, field_name)),
        _ => false,
    }
}

fn build_local_only_aggregation_output(
    strategy: &LocalAggregationStrategy,
    group_stages: &[NormalizedShardStage],
) -> LocalAggregationOutputBuild {
    match strategy {
        LocalAggregationStrategy::DigestOnly => LocalAggregationOutputBuild {
            output: None,
            issues: Vec::new(),
        },
        LocalAggregationStrategy::CollectOutputs => LocalAggregationOutputBuild {
            output: Some(serde_json::Value::Array(
                group_stages
                    .iter()
                    .map(|stage| {
                        serde_json::json!({
                            "stage_id": &stage.stage_id,
                            "shard_id": &stage.shard_id,
                            "provider": &stage.provider,
                            "output": stage
                                .dispatch_output
                                .as_deref()
                                .map(parse_local_output_value)
                                .unwrap_or(serde_json::Value::Null),
                        })
                    })
                    .collect::<Vec<_>>(),
            )),
            issues: missing_stage_output_issues(group_stages),
        },
        LocalAggregationStrategy::JsonObjectByShard => {
            let issues = missing_stage_output_issues(group_stages);
            let mut object = serde_json::Map::new();
            for stage in group_stages {
                let Some(output) = stage.dispatch_output.as_deref() else {
                    continue;
                };
                let key = stage
                    .shard_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(stage.stage_id.as_str())
                    .to_string();
                object.insert(key, parse_local_output_value(output));
            }
            LocalAggregationOutputBuild {
                output: Some(serde_json::Value::Object(object)),
                issues,
            }
        }
        LocalAggregationStrategy::TextConcatenate => {
            let issues = missing_stage_output_issues(group_stages);
            let text = group_stages
                .iter()
                .filter_map(|stage| stage.dispatch_output.as_deref())
                .collect::<Vec<_>>()
                .join("\n");
            LocalAggregationOutputBuild {
                output: Some(serde_json::Value::String(text)),
                issues,
            }
        }
    }
}

fn parse_local_output_value(output: &str) -> serde_json::Value {
    serde_json::from_str(output).unwrap_or_else(|_| serde_json::Value::String(output.to_string()))
}

fn missing_stage_output_issues(group_stages: &[NormalizedShardStage]) -> Vec<String> {
    group_stages
        .iter()
        .filter(|stage| stage.dispatch_output.is_none())
        .map(|stage| format!("stage_id={} missing dispatch output", stage.stage_id))
        .collect::<Vec<_>>()
}

struct LocalAggregationOutputBuild {
    output: Option<serde_json::Value>,
    issues: Vec<String>,
}

#[derive(Clone)]
struct NormalizedShardStage {
    stage_id: String,
    provider: String,
    shard_group: Option<String>,
    shard_id: Option<String>,
    external_view_digest: String,
    dispatch_allowed: bool,
    executed: bool,
    dispatched: bool,
    dispatch_output_digest: Option<String>,
    dispatch_output: Option<String>,
}

fn task_contract_dispatch_block_reason(policy: &Policy, task_profile: &str) -> Option<String> {
    let assessment = assess_task_contract(policy, task_profile);
    if assessment.dispatch_allowed {
        None
    } else {
        Some(assessment.issues.join("; "))
    }
}

fn adapter_capability_block_reason(
    policy: &Policy,
    task_profile: &str,
    adapter_capabilities: &AdapterCapabilities,
) -> Option<String> {
    let issues = policy.adapter_contract_issues(task_profile, &adapter_capabilities.adapter_class);
    if issues.is_empty() {
        None
    } else {
        Some(issues.join("; "))
    }
}

async fn restore_output(
    State(state): State<AppState>,
    Json(input): Json<OutputRestoreRequest>,
) -> Result<Json<OutputRestoreResponse>, ApiError> {
    let mappings = load_mappings_for_audit(&state, input.audit_id)?;
    let mut restored = input.output;
    let mut replacements = 0usize;
    for mapping in mappings {
        if restored.contains(&mapping.token) {
            restored = restored.replace(&mapping.token, &mapping.original_value);
            replacements += 1;
        }
    }

    Ok(Json(OutputRestoreResponse {
        audit_id: input.audit_id,
        restored_output: restored,
        replacements,
    }))
}

async fn review_status(
    State(state): State<AppState>,
    Json(input): Json<ReviewStatusRequest>,
) -> Result<Json<ManualReviewState>, ApiError> {
    Ok(Json(
        load_manual_review_status(&state, input.audit_id).await?,
    ))
}

async fn review_approve(
    State(state): State<AppState>,
    Json(input): Json<ReviewDecisionRequest>,
) -> Result<Json<ManualReviewState>, ApiError> {
    let status = set_manual_review_decision(&state, input, ReviewStatus::Approved).await?;
    persist_review_event(&state, "approved", &status).await?;
    Ok(Json(status))
}

async fn review_reject(
    State(state): State<AppState>,
    Json(input): Json<ReviewDecisionRequest>,
) -> Result<Json<ManualReviewState>, ApiError> {
    let status = set_manual_review_decision(&state, input, ReviewStatus::Rejected).await?;
    persist_review_event(&state, "rejected", &status).await?;
    Ok(Json(status))
}

fn parse_review_mode(value: &str) -> Result<ReviewMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "off" | "disabled" | "none" | "false" | "0" => Ok(ReviewMode::Off),
        "manual" | "required" | "human" | "human_review" => Ok(ReviewMode::Manual),
        other => Err(anyhow::anyhow!(
            "invalid PRIVAGATE_REVIEW_MODE={other}; use off or manual"
        )),
    }
}

async fn create_manual_review_if_required(
    state: &AppState,
    output: &mut GatewayOutput,
) -> Result<Option<ManualReviewState>> {
    let review = create_manual_review_for_digest_if_required(
        state,
        output.audit_summary.audit_id,
        &output.audit_summary.external_view_digest,
        "manual review required before external dispatch",
    )
    .await?;
    if review.is_some() {
        output.audit_summary.blocked = true;
    }
    Ok(review)
}

async fn create_manual_review_for_digest_if_required(
    state: &AppState,
    audit_id: Uuid,
    external_view_digest: &str,
    reason: &str,
) -> Result<Option<ManualReviewState>> {
    if state.review_mode != ReviewMode::Manual {
        return Ok(None);
    }

    let now = Utc::now();
    let record = ReviewRecord {
        audit_id,
        external_view_digest: external_view_digest.to_string(),
        status: ReviewStatus::Pending,
        reviewer: None,
        reason: Some(reason.to_string()),
        created_at: now,
        updated_at: now,
    };
    let status = ManualReviewState::from(&record);
    persist_review_record(state, &record).await?;
    Ok(Some(status))
}

async fn apply_projection_dispatch_controls(
    state: &AppState,
    output: &mut GatewayOutput,
    task_contract_assessment: &TaskContractAssessment,
) -> Result<Option<ManualReviewState>> {
    if !task_contract_assessment.dispatch_allowed {
        output.audit_summary.blocked = true;
        return Ok(None);
    }

    create_manual_review_if_required(state, output).await
}

async fn load_manual_review_status(state: &AppState, audit_id: Uuid) -> Result<ManualReviewState> {
    let record = load_review_record(state, audit_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("manual review record not found for audit_id={audit_id}"))?;
    Ok(ManualReviewState::from(&record))
}

async fn set_manual_review_decision(
    state: &AppState,
    input: ReviewDecisionRequest,
    status: ReviewStatus,
) -> Result<ManualReviewState> {
    let reviewer = input.reviewer.trim();
    if reviewer.is_empty() {
        return Err(anyhow::anyhow!("reviewer must not be empty"));
    }

    let mut record = load_review_record(state, input.audit_id)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "manual review record not found for audit_id={}",
                input.audit_id
            )
        })?;
    record.status = status;
    record.reviewer = Some(reviewer.to_string());
    record.reason = input.reason.filter(|reason| !reason.trim().is_empty());
    record.updated_at = Utc::now();
    persist_review_record(state, &record).await?;
    Ok(ManualReviewState::from(&record))
}

async fn review_dispatch_block_reason(
    state: &AppState,
    audit_id: Option<Uuid>,
    external_view_digest: &str,
) -> Result<Option<String>> {
    if state.review_mode != ReviewMode::Manual {
        return Ok(None);
    }

    let Some(audit_id) = audit_id else {
        return Ok(Some(
            "missing audit_id for manually reviewed projection".to_string(),
        ));
    };

    let Some(record) = load_review_record(state, audit_id).await? else {
        return Ok(Some(format!(
            "no manual review record for audit_id={audit_id}"
        )));
    };

    if record.external_view_digest != external_view_digest {
        return Ok(Some(format!(
            "external_view_digest mismatch for audit_id={audit_id}"
        )));
    }

    match record.status {
        ReviewStatus::Approved => Ok(None),
        ReviewStatus::Pending => Ok(Some(format!(
            "audit_id={audit_id} is pending manual review"
        ))),
        ReviewStatus::Rejected => Ok(Some(format!(
            "audit_id={audit_id} was rejected by manual review"
        ))),
    }
}

async fn persist_review_record(state: &AppState, record: &ReviewRecord) -> Result<()> {
    let store = state.review_store.as_ref().clone();
    match store {
        ReviewStore::Jsonl { path } => persist_jsonl_review_record(&path, record),
        ReviewStore::Postgres { connection_string } => {
            persist_postgres_review_record(&connection_string, record).await
        }
    }
}

async fn load_review_record(state: &AppState, audit_id: Uuid) -> Result<Option<ReviewRecord>> {
    let store = state.review_store.as_ref().clone();
    match store {
        ReviewStore::Jsonl { path } => load_jsonl_review_record(&path, audit_id),
        ReviewStore::Postgres { connection_string } => {
            load_postgres_review_record(&connection_string, audit_id).await
        }
    }
}

fn persist_jsonl_review_record(path: &PathBuf, record: &ReviewRecord) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open manual review log: {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(record)?)?;
    Ok(())
}

fn load_jsonl_review_record(path: &PathBuf, audit_id: Uuid) -> Result<Option<ReviewRecord>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read manual review log: {}", path.display()))?;
    let mut latest = None;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let record: ReviewRecord = serde_json::from_str(line)
            .with_context(|| format!("invalid manual review row in {}", path.display()))?;
        if record.audit_id == audit_id {
            latest = Some(record);
        }
    }
    Ok(latest)
}

async fn persist_postgres_review_record(
    connection_string: &str,
    record: &ReviewRecord,
) -> Result<()> {
    let (client, connection) = tokio_postgres::connect(connection_string, tokio_postgres::NoTls)
        .await
        .context("failed to connect PostgreSQL manual review store")?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::error!(%error, "PostgreSQL manual review connection error");
        }
    });
    client
        .batch_execute(CREATE_POSTGRES_MANUAL_REVIEW_TABLE_SQL)
        .await
        .context("failed to ensure PostgreSQL manual review table")?;
    client
        .execute(
            UPSERT_POSTGRES_MANUAL_REVIEW_SQL,
            &[
                &record.audit_id.to_string(),
                &record.external_view_digest,
                &record.status.to_string(),
                &record.reviewer,
                &record.reason,
                &record.created_at.to_rfc3339(),
                &record.updated_at.to_rfc3339(),
            ],
        )
        .await
        .context("failed to upsert PostgreSQL manual review row")?;
    Ok(())
}

async fn load_postgres_review_record(
    connection_string: &str,
    audit_id: Uuid,
) -> Result<Option<ReviewRecord>> {
    let (client, connection) = tokio_postgres::connect(connection_string, tokio_postgres::NoTls)
        .await
        .context("failed to connect PostgreSQL manual review store")?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::error!(%error, "PostgreSQL manual review connection error");
        }
    });
    client
        .batch_execute(CREATE_POSTGRES_MANUAL_REVIEW_TABLE_SQL)
        .await
        .context("failed to ensure PostgreSQL manual review table")?;
    let row = client
        .query_opt(SELECT_POSTGRES_MANUAL_REVIEW_SQL, &[&audit_id.to_string()])
        .await
        .context("failed to load PostgreSQL manual review row")?;

    let Some(row) = row else {
        return Ok(None);
    };

    let audit_id: String = row.get(0);
    let created_at: String = row.get(5);
    let updated_at: String = row.get(6);
    Ok(Some(ReviewRecord {
        audit_id: Uuid::parse_str(&audit_id)
            .with_context(|| format!("invalid PostgreSQL audit_id value: {audit_id}"))?,
        external_view_digest: row.get(1),
        status: parse_review_status(&row.get::<_, String>(2))?,
        reviewer: row.get(3),
        reason: row.get(4),
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .with_context(|| format!("invalid PostgreSQL created_at value: {created_at}"))?
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)
            .with_context(|| format!("invalid PostgreSQL updated_at value: {updated_at}"))?
            .with_timezone(&Utc),
    }))
}

fn parse_review_status(value: &str) -> Result<ReviewStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "pending" => Ok(ReviewStatus::Pending),
        "approved" => Ok(ReviewStatus::Approved),
        "rejected" => Ok(ReviewStatus::Rejected),
        other => Err(anyhow::anyhow!("invalid review status: {other}")),
    }
}

fn persist_local_mappings(state: &AppState, output: &GatewayOutput) -> Result<()> {
    if output.local_mappings.is_empty() {
        return Ok(());
    }

    let guard = state
        .mapping_log
        .lock()
        .map_err(|_| anyhow::anyhow!("mapping log lock poisoned"))?;
    if let Some(parent) = guard.path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&guard.path)
        .with_context(|| format!("failed to open local mapping log: {}", guard.path.display()))?;

    for mapping in &output.local_mappings {
        let row = serde_json::json!({
            "audit_id": output.audit_summary.audit_id,
            "policy_version": &output.audit_summary.policy_version,
            "external_view_digest": &output.audit_summary.external_view_digest,
            "field_name": &mapping.field_name,
            "field_type": &mapping.field_type,
            "token": &mapping.token,
            "original_value": &mapping.original_value,
        });
        writeln!(file, "{}", serde_json::to_string(&row)?)?;
    }
    Ok(())
}

async fn persist_audit_summary(
    state: &AppState,
    output: &GatewayOutput,
    task_contract_assessment: Option<&TaskContractAssessment>,
) -> Result<()> {
    let row = serde_json::json!({
        "audit_summary": &output.audit_summary,
        "task_profile": &output.utility_report.task_profile,
        "task_contract_assessment": task_contract_assessment,
        "privacy_report_id": output.privacy_report.report_id,
        "utility_report_id": output.utility_report.report_id,
        "privacy_verification_results": &output.privacy_report.verification_results,
        "utility_constraint_results": &output.utility_report.constraint_results,
        "task_loss_bounds": &output.utility_report.task_loss_bounds,
    });
    persist_audit_row(state, &row).await
}

async fn persist_statistic_audit(state: &AppState, output: &StatisticOutput) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "statistic_proof",
        "input_digest": &output.input_digest,
        "privacy_budget": &output.privacy_budget,
        "results": &output.results,
        "verification_results": &output.verification_results,
    });
    persist_audit_row(state, &row).await
}

async fn persist_review_event(
    state: &AppState,
    event: &str,
    review: &ManualReviewState,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "manual_review_gate",
        "event": event,
        "manual_review": review,
    });
    persist_audit_row(state, &row).await
}

async fn persist_policy_block_event(
    state: &AppState,
    output: &GatewayOutput,
    task_contract_assessment: &TaskContractAssessment,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "task_contract_gate",
        "event": "blocked",
        "audit_id": output.audit_summary.audit_id,
        "external_view_digest": &output.audit_summary.external_view_digest,
        "task_profile": &task_contract_assessment.task_profile,
        "issues": &task_contract_assessment.issues,
    });
    persist_audit_row(state, &row).await
}

async fn persist_dispatch_policy_block_event(
    state: &AppState,
    audit_id: Option<Uuid>,
    external_view_digest: &str,
    provider: &str,
    adapter_capabilities: &AdapterCapabilities,
    task_contract_assessment: &TaskContractAssessment,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "task_contract_gate",
        "event": "blocked_dispatch",
        "audit_id": audit_id,
        "external_view_digest": external_view_digest,
        "provider": provider,
        "adapter_capabilities": adapter_capabilities,
        "task_profile": &task_contract_assessment.task_profile,
        "issues": &task_contract_assessment.issues,
    });
    persist_audit_row(state, &row).await
}

async fn persist_adapter_capability_block_event(
    state: &AppState,
    audit_id: Option<Uuid>,
    external_view_digest: &str,
    provider: &str,
    adapter_capabilities: &AdapterCapabilities,
    task_profile: &str,
    reason: &str,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "adapter_capability_gate",
        "event": "blocked_dispatch",
        "audit_id": audit_id,
        "external_view_digest": external_view_digest,
        "provider": provider,
        "adapter_capabilities": adapter_capabilities,
        "task_profile": task_profile,
        "reason": reason,
    });
    persist_audit_row(state, &row).await
}

async fn persist_route_plan_validation_event(
    state: &AppState,
    route_plan: &RoutePlanValidationResponse,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "route_plan_evidence",
        "event": "validated",
        "route_id": &route_plan.route_id,
        "aggregation_strategy": &route_plan.aggregation_strategy,
        "residual_risk_notes": &route_plan.residual_risk_notes,
        "dispatch_allowed": route_plan.dispatch_allowed,
        "stages": &route_plan.stages,
    });
    persist_audit_row(state, &row).await
}

async fn persist_route_plan_execution_event(
    state: &AppState,
    route_plan: &RoutePlanExecutionResponse,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "route_plan_evidence",
        "event": "executed",
        "route_id": &route_plan.route_id,
        "aggregation_strategy": &route_plan.aggregation_strategy,
        "residual_risk_notes": &route_plan.residual_risk_notes,
        "dispatch_allowed": route_plan.dispatch_allowed,
        "stop_on_block": route_plan.stop_on_block,
        "halted": route_plan.halted,
        "executed_stage_count": route_plan.executed_stage_count,
        "dispatched_stage_count": route_plan.dispatched_stage_count,
        "all_stages_dispatched": route_plan.all_stages_dispatched,
        "runtime_adapter_capabilities": &route_plan.runtime_adapter_capabilities,
        "stages": &route_plan.stages,
    });
    persist_audit_row(state, &row).await
}

async fn persist_shard_plan_validation_event(
    state: &AppState,
    shard_plan: &ShardPlanValidationResponse,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "shard_plan_evidence",
        "event": "validated",
        "route_plan": &shard_plan.route_plan,
        "aggregation_rules": &shard_plan.aggregation_rules,
        "local_aggregation_summary": &shard_plan.local_aggregation_summary,
    });
    persist_audit_row(state, &row).await
}

async fn persist_shard_plan_execution_event(
    state: &AppState,
    shard_plan: &ShardPlanExecutionResponse,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "shard_plan_evidence",
        "event": "executed",
        "route_plan_execution": &shard_plan.route_plan_execution,
        "aggregation_rules": &shard_plan.aggregation_rules,
        "local_aggregation_summary": &shard_plan.local_aggregation_summary,
    });
    persist_audit_row(state, &row).await
}

async fn persist_shard_promotion_binding_event(
    state: &AppState,
    response: &ShardPromotionBindingResponse,
) -> Result<()> {
    let row = serde_json::json!({
        "report_type": "promotion_binding_evidence",
        "event": if response.binding_created { "bound" } else { "blocked" },
        "route_id": &response.route_id,
        "group_id": &response.group_id,
        "binding_created": response.binding_created,
        "ready_for_follow_up_route": response.ready_for_follow_up_route,
        "issues": &response.issues,
        "source_local_aggregation_digest": &response.source_local_aggregation_digest,
        "source_local_only_output_digest": &response.source_local_only_output_digest,
        "promotion": &response.promotion,
        "follow_up_binding": &response.follow_up_binding,
        "manual_review": &response.manual_review,
    });
    persist_audit_row(state, &row).await
}

async fn persist_projection_control_events(
    state: &AppState,
    output: &GatewayOutput,
    task_contract_assessment: &TaskContractAssessment,
    manual_review: Option<&ManualReviewState>,
) -> Result<()> {
    if !task_contract_assessment.dispatch_allowed {
        persist_policy_block_event(state, output, task_contract_assessment).await?;
    }
    if let Some(review) = manual_review {
        persist_review_event(state, "pending", review).await?;
    }
    Ok(())
}

async fn persist_audit_row(state: &AppState, row: &serde_json::Value) -> Result<()> {
    let sink = {
        let guard = state
            .audit_sink
            .lock()
            .map_err(|_| anyhow::anyhow!("audit sink lock poisoned"))?;
        match &*guard {
            AuditSink::Jsonl { path } => AuditSink::Jsonl { path: path.clone() },
            AuditSink::Postgres { connection_string } => AuditSink::Postgres {
                connection_string: connection_string.clone(),
            },
        }
    };

    match sink {
        AuditSink::Jsonl { path } => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .with_context(|| format!("failed to open audit log: {}", path.display()))?;
            writeln!(file, "{}", serde_json::to_string(row)?)?;
            Ok(())
        }
        AuditSink::Postgres { connection_string } => {
            persist_postgres_audit(&connection_string, row).await
        }
    }
}

async fn persist_postgres_audit(connection_string: &str, row: &serde_json::Value) -> Result<()> {
    let (client, connection) = tokio_postgres::connect(connection_string, tokio_postgres::NoTls)
        .await
        .context("failed to connect PostgreSQL audit sink")?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::error!(%error, "PostgreSQL audit connection error");
        }
    });
    client
        .batch_execute(CREATE_POSTGRES_AUDIT_TABLE_SQL)
        .await
        .context("failed to ensure PostgreSQL audit table")?;
    client
        .execute(INSERT_POSTGRES_AUDIT_ROW_SQL, &[row])
        .await
        .context("failed to append PostgreSQL audit row")?;
    Ok(())
}

fn load_mappings_for_audit(state: &AppState, audit_id: Uuid) -> Result<Vec<PersistedMapping>> {
    let guard = state
        .mapping_log
        .lock()
        .map_err(|_| anyhow::anyhow!("mapping log lock poisoned"))?;
    if !guard.path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&guard.path)
        .with_context(|| format!("failed to read local mapping log: {}", guard.path.display()))?;
    let mut mappings = Vec::new();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let mapping: PersistedMapping = serde_json::from_str(line)
            .with_context(|| format!("invalid mapping log row in {}", guard.path.display()))?;
        if mapping.audit_id == audit_id {
            mappings.push(mapping);
        }
    }
    Ok(mappings)
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct ProjectResponse {
    #[serde(flatten)]
    output: GatewayOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_review: Option<ManualReviewState>,
    task_contract_assessment: TaskContractAssessment,
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectRequest {
    #[serde(flatten)]
    input: GatewayInput,
    #[serde(default)]
    task_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ManualReviewState {
    required: bool,
    audit_id: Uuid,
    external_view_digest: String,
    status: ReviewStatus,
    reviewer: Option<String>,
    reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskContractAssessment {
    task_profile: String,
    dispatch_allowed: bool,
    issues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RoutePlanValidationRequest {
    #[serde(default)]
    route_id: Option<String>,
    #[serde(default)]
    aggregation_strategy: Option<String>,
    #[serde(default)]
    residual_risk_notes: Vec<String>,
    stages: Vec<RouteStageRequest>,
}

#[derive(Debug, Clone, Deserialize)]
struct RoutePlanExecutionRequest {
    #[serde(flatten)]
    route_plan: RoutePlanValidationRequest,
    #[serde(default = "default_true")]
    stop_on_block: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ShardPlanValidationRequest {
    #[serde(flatten)]
    route_plan: RoutePlanValidationRequest,
    #[serde(default)]
    aggregation_rules: ShardAggregationRules,
}

#[derive(Debug, Clone, Deserialize)]
struct ShardPlanExecutionRequest {
    #[serde(flatten)]
    route_plan_execution: RoutePlanExecutionRequest,
    #[serde(default)]
    aggregation_rules: ShardAggregationRules,
}

#[derive(Debug, Clone, Deserialize)]
struct ShardPromotionBindingRequest {
    route_plan_execution: RoutePlanExecutionResponse,
    aggregation_rules: ShardAggregationRules,
    group_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RouteStageRequest {
    stage_id: String,
    provider: String,
    task_profile: String,
    adapter_class: privagate_core::AdapterClass,
    audit_id: Option<Uuid>,
    #[serde(default)]
    shard_group: Option<String>,
    #[serde(default)]
    shard_id: Option<String>,
    external_view: privagate_core::ExternalView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoutePlanValidationResponse {
    route_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    aggregation_strategy: Option<String>,
    residual_risk_notes: Vec<String>,
    dispatch_allowed: bool,
    stages: Vec<RouteStageValidation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ShardAggregationRules {
    #[serde(default = "default_true")]
    require_shard_metadata: bool,
    #[serde(default)]
    require_distinct_providers: bool,
    #[serde(default)]
    strategy: LocalAggregationStrategy,
    #[serde(default)]
    promotion: AggregationPromotionRules,
    #[serde(default)]
    expected_groups: Vec<ShardGroupExpectation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShardGroupExpectation {
    group_id: String,
    min_shards: usize,
    #[serde(default)]
    strategy: Option<LocalAggregationStrategy>,
    #[serde(default)]
    promotion: Option<AggregationPromotionRules>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LocalAggregationStrategy {
    #[default]
    DigestOnly,
    CollectOutputs,
    JsonObjectByShard,
    TextConcatenate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AggregationPromotionRules {
    #[serde(default)]
    mode: AggregationPromotionMode,
    #[serde(default)]
    candidate_task_profile: Option<String>,
    #[serde(default)]
    utility_verification: PromotionUtilityVerificationRules,
    #[serde(default)]
    allowed_content_types: Vec<privagate_core::transform::ContentType>,
    #[serde(default)]
    max_serialized_bytes: Option<usize>,
    #[serde(default)]
    max_text_chars: Option<usize>,
    #[serde(default)]
    max_array_items: Option<usize>,
    #[serde(default)]
    max_object_keys: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AggregationPromotionMode {
    #[default]
    LocalOnly,
    ExternalViewCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromotionUtilityVerificationRules {
    #[serde(default = "default_true")]
    require_required_fields: bool,
    #[serde(default)]
    verify_constraint_results: bool,
}

impl Default for PromotionUtilityVerificationRules {
    fn default() -> Self {
        Self {
            require_required_fields: true,
            verify_constraint_results: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoutePlanExecutionResponse {
    route_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    aggregation_strategy: Option<String>,
    residual_risk_notes: Vec<String>,
    dispatch_allowed: bool,
    stop_on_block: bool,
    halted: bool,
    executed_stage_count: usize,
    dispatched_stage_count: usize,
    all_stages_dispatched: bool,
    runtime_adapter_capabilities: AdapterCapabilities,
    stages: Vec<RouteStageExecution>,
}

#[derive(Debug, Clone, Serialize)]
struct ShardPlanValidationResponse {
    route_plan: RoutePlanValidationResponse,
    aggregation_rules: ShardAggregationRules,
    local_aggregation_summary: LocalAggregationSummary,
}

#[derive(Debug, Clone, Serialize)]
struct ShardPlanExecutionResponse {
    route_plan_execution: RoutePlanExecutionResponse,
    aggregation_rules: ShardAggregationRules,
    local_aggregation_summary: LocalAggregationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RouteStageValidation {
    stage_id: String,
    provider: String,
    task_profile: String,
    adapter_class: privagate_core::AdapterClass,
    audit_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shard_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shard_id: Option<String>,
    external_view_digest: String,
    blocked_by_policy: bool,
    blocked_by_review: bool,
    dispatch_allowed: bool,
    issues: Vec<String>,
    task_contract_assessment: TaskContractAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RouteStageExecution {
    #[serde(flatten)]
    validation: RouteStageValidation,
    executed: bool,
    runtime_issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skipped_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dispatch_output_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dispatch_response: Option<ModelDispatchResponse>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalAggregationSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    aggregation_strategy: Option<String>,
    ready_for_local_aggregation: bool,
    issue_count: usize,
    issues: Vec<String>,
    groups: Vec<LocalAggregationGroupSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalAggregationGroupSummary {
    group_id: String,
    aggregation_strategy: LocalAggregationStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_min_shards: Option<usize>,
    observed_stage_count: usize,
    unique_provider_count: usize,
    dispatch_allowed_stage_count: usize,
    executed_stage_count: usize,
    dispatched_stage_count: usize,
    all_stage_outputs_bound: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_aggregation_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_only_output_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_only_output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    promotion: Option<LocalPromotionSummary>,
    stage_ids: Vec<String>,
    shard_ids: Vec<String>,
    output_digests: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalPromotionSummary {
    mode: AggregationPromotionMode,
    promotion_allowed: bool,
    ready_for_follow_up_route: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_task_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_content_type: Option<privagate_core::transform::ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_contract_assessment: Option<TaskContractAssessment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    utility_assessment: Option<FollowUpUtilityAssessment>,
    issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_view_candidate_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_view_candidate: Option<privagate_core::ExternalView>,
}

#[derive(Debug, Clone, Serialize)]
struct FollowUpUtilityAssessment {
    task_profile: String,
    verification_passed: bool,
    required_field_presence: privagate_core::report::PreservationMetric,
    missing_required_fields: Vec<String>,
    constraint_results: Vec<privagate_core::report::VerificationResult>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ShardPromotionBindingResponse {
    route_id: String,
    group_id: String,
    binding_created: bool,
    ready_for_follow_up_route: bool,
    issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_local_aggregation_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_local_only_output_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    promotion: Option<LocalPromotionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    follow_up_binding: Option<FollowUpViewBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_review: Option<ManualReviewState>,
}

#[derive(Debug, Clone, Serialize)]
struct FollowUpViewBinding {
    task_profile: String,
    audit_summary: privagate_core::report::AuditSummary,
    external_view: privagate_core::ExternalView,
    utility_assessment: FollowUpUtilityAssessment,
}

impl From<&ReviewRecord> for ManualReviewState {
    fn from(record: &ReviewRecord) -> Self {
        Self {
            required: true,
            audit_id: record.audit_id,
            external_view_digest: record.external_view_digest.clone(),
            status: record.status.clone(),
            reviewer: record.reviewer.clone(),
            reason: record.reason.clone(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            ReviewStatus::Pending => "pending",
            ReviewStatus::Approved => "approved",
            ReviewStatus::Rejected => "rejected",
        };
        formatter.write_str(status)
    }
}

fn project_policy(base_policy: &Policy, requested_task_profile: Option<&str>) -> Policy {
    let mut policy = base_policy.clone();
    if let Some(task_profile) = requested_task_profile
        .map(str::trim)
        .filter(|task_profile| !task_profile.is_empty())
    {
        policy.task_profile = task_profile.to_string();
    }
    policy
}

fn assess_task_contract(policy: &Policy, task_profile: &str) -> TaskContractAssessment {
    let issues = policy
        .task_contract_issues(task_profile)
        .into_iter()
        .map(|issue| issue.details)
        .collect::<Vec<_>>();
    TaskContractAssessment {
        task_profile: task_profile.to_string(),
        dispatch_allowed: issues.is_empty(),
        issues,
    }
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct ReviewStatusRequest {
    audit_id: Uuid,
}

#[derive(Deserialize)]
struct ReviewDecisionRequest {
    audit_id: Uuid,
    reviewer: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct OutputInspectionRequest {
    audit_id: Uuid,
    output: String,
}

#[derive(Deserialize)]
struct OutputRestoreRequest {
    audit_id: Uuid,
    output: String,
}

#[derive(Serialize)]
struct OutputRestoreResponse {
    audit_id: Uuid,
    restored_output: String,
    replacements: usize,
}

#[derive(Deserialize)]
struct RagChunkProjectRequest {
    #[serde(default)]
    task_profile: Option<String>,
    chunks: Vec<RagChunkInput>,
}

#[derive(Deserialize)]
struct RagChunkInput {
    chunk_id: String,
    source_uri: String,
    content_type: privagate_core::transform::ContentType,
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct RagChunkProjectResponse {
    chunks: Vec<RagChunkProjection>,
}

#[derive(Serialize)]
struct RagChunkProjection {
    chunk_id: String,
    source_uri: String,
    external_view_digest: String,
    audit_id: Uuid,
    external_view: privagate_core::ExternalView,
    privacy_passed: bool,
    utility_passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_review: Option<ManualReviewState>,
    task_contract_assessment: TaskContractAssessment,
}

#[derive(Deserialize)]
struct ToolInspectionRequest {
    tool_name: String,
    audit_ids: Vec<Uuid>,
    input: String,
    output: String,
}

#[derive(Serialize)]
struct ToolInspectionResponse {
    tool_name: String,
    passed: bool,
    unauthorized_sensitive_count: usize,
    findings: Vec<ToolLeakFinding>,
}

#[derive(Serialize)]
struct ToolLeakFinding {
    audit_id: Uuid,
    location: String,
    field_name: String,
    field_type: String,
    token: String,
}

#[derive(Deserialize)]
struct SessionRiskRequest {
    session_id: String,
    risk_bound: f64,
    events: Vec<SessionRiskEvent>,
}

#[derive(Deserialize)]
struct SessionRiskEvent {
    external_view_digest: Option<String>,
    privacy_budget_epsilon: Option<f64>,
}

#[derive(Serialize)]
struct SessionRiskResponse {
    session_id: String,
    event_count: usize,
    exposure_events: usize,
    epsilon_total: f64,
    risk_score: f64,
    risk_bound: f64,
    passed: bool,
}

#[derive(Serialize)]
struct OutputInspectionResponse {
    audit_id: Uuid,
    passed: bool,
    unauthorized_sensitive_output_count: usize,
    findings: Vec<OutputLeakFinding>,
}

#[derive(Serialize)]
struct OutputLeakFinding {
    field_name: String,
    field_type: String,
    token: String,
}

#[derive(Deserialize)]
struct PersistedMapping {
    audit_id: Uuid,
    field_name: String,
    field_type: String,
    token: String,
    original_value: String,
}

#[derive(Debug)]
struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "privagate_error",
            "message": self.0.to_string()
        });
        (StatusCode::BAD_REQUEST, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privagate_core::policy::ConstraintPolicy;
    use std::collections::BTreeMap;

    fn unique_test_file(prefix: &str) -> PathBuf {
        PathBuf::from("target").join(format!("{prefix}-{}.jsonl", Uuid::new_v4()))
    }

    fn base_policy(task_profile: &str) -> Policy {
        Policy {
            policy_version: "test".to_string(),
            task_profile: task_profile.to_string(),
            key_domain: "local/test".to_string(),
            fields: BTreeMap::new(),
            task_contracts: BTreeMap::new(),
            constraints: ConstraintPolicy::default(),
            statistics: Vec::new(),
        }
    }

    fn test_state(review_mode: ReviewMode) -> AppState {
        test_state_with_policy_and_review_log_and_adapter(
            review_mode,
            base_policy("manual_review_test"),
            unique_test_file("test-manual-review"),
            Arc::new(DisabledModelAdapter),
        )
    }

    fn test_state_with_policy(review_mode: ReviewMode, policy: Policy) -> AppState {
        test_state_with_policy_and_review_log_and_adapter(
            review_mode,
            policy,
            unique_test_file("test-review"),
            Arc::new(DisabledModelAdapter),
        )
    }

    fn test_state_with_policy_and_review_log(
        review_mode: ReviewMode,
        policy: Policy,
        review_log_path: PathBuf,
    ) -> AppState {
        test_state_with_policy_and_review_log_and_adapter(
            review_mode,
            policy,
            review_log_path,
            Arc::new(DisabledModelAdapter),
        )
    }

    fn test_state_with_policy_and_adapter(
        review_mode: ReviewMode,
        policy: Policy,
        model_adapter: Arc<dyn ModelAdapter>,
    ) -> AppState {
        test_state_with_policy_and_review_log_and_adapter(
            review_mode,
            policy,
            unique_test_file("test-review"),
            model_adapter,
        )
    }

    fn test_state_with_policy_and_review_log_and_adapter(
        review_mode: ReviewMode,
        policy: Policy,
        review_log_path: PathBuf,
        model_adapter: Arc<dyn ModelAdapter>,
    ) -> AppState {
        AppState {
            policy: Arc::new(policy),
            hmac_key: Arc::new(b"test-secret".to_vec()),
            mapping_log: Arc::new(Mutex::new(MappingLog {
                path: unique_test_file("test-mappings"),
            })),
            audit_sink: Arc::new(Mutex::new(AuditSink::Jsonl {
                path: unique_test_file("test-audit"),
            })),
            model_adapter,
            review_mode,
            review_store: Arc::new(ReviewStore::Jsonl {
                path: review_log_path,
            }),
        }
    }

    #[test]
    fn parses_manual_review_mode() {
        assert_eq!(parse_review_mode("off").unwrap(), ReviewMode::Off);
        assert_eq!(parse_review_mode("manual").unwrap(), ReviewMode::Manual);
        assert!(parse_review_mode("maybe").is_err());
    }

    #[tokio::test]
    async fn manual_review_blocks_until_approved_for_same_digest() {
        let state = test_state(ReviewMode::Manual);
        let input = GatewayInput {
            content_type: privagate_core::transform::ContentType::Text,
            payload: serde_json::json!("synthetic public text"),
        };
        let mut output = process_document(input, &state.policy, state.hmac_key.as_ref()).unwrap();
        let digest = output.audit_summary.external_view_digest.clone();

        let review = create_manual_review_if_required(&state, &mut output)
            .await
            .unwrap()
            .unwrap();
        assert!(output.audit_summary.blocked);

        let pending_reason = review_dispatch_block_reason(&state, Some(review.audit_id), &digest)
            .await
            .unwrap();
        assert!(pending_reason
            .expect("pending review should block")
            .contains("pending manual review"));

        set_manual_review_decision(
            &state,
            ReviewDecisionRequest {
                audit_id: review.audit_id,
                reviewer: "unit-test-reviewer".to_string(),
                reason: Some("approved synthetic projection".to_string()),
            },
            ReviewStatus::Approved,
        )
        .await
        .unwrap();

        assert!(
            review_dispatch_block_reason(&state, Some(review.audit_id), &digest)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            review_dispatch_block_reason(&state, Some(review.audit_id), "sha256:changed")
                .await
                .unwrap()
                .expect("digest mismatch should block")
                .contains("mismatch")
        );
    }

    #[tokio::test]
    async fn manual_review_persists_across_state_reloads() {
        let review_log_path = unique_test_file("persisted-manual-review");
        let input = GatewayInput {
            content_type: privagate_core::transform::ContentType::Text,
            payload: serde_json::json!("synthetic durable review text"),
        };

        let mut initial_output = {
            let state = test_state_with_policy_and_review_log(
                ReviewMode::Manual,
                base_policy("manual_review_test"),
                review_log_path.clone(),
            );
            let mut output =
                process_document(input.clone(), &state.policy, state.hmac_key.as_ref()).unwrap();
            let review = create_manual_review_if_required(&state, &mut output)
                .await
                .unwrap()
                .expect("pending review should be created");
            assert_eq!(review.status, ReviewStatus::Pending);
            output
        };
        let audit_id = initial_output.audit_summary.audit_id;
        let digest = initial_output.audit_summary.external_view_digest.clone();

        let state = test_state_with_policy_and_review_log(
            ReviewMode::Manual,
            base_policy("manual_review_test"),
            review_log_path.clone(),
        );
        assert!(
            review_dispatch_block_reason(&state, Some(audit_id), &digest)
                .await
                .unwrap()
                .expect("pending review should still block after reload")
                .contains("pending manual review")
        );

        set_manual_review_decision(
            &state,
            ReviewDecisionRequest {
                audit_id,
                reviewer: "durability-reviewer".to_string(),
                reason: Some("approved after reload".to_string()),
            },
            ReviewStatus::Approved,
        )
        .await
        .unwrap();

        let reloaded_state = test_state_with_policy_and_review_log(
            ReviewMode::Manual,
            base_policy("manual_review_test"),
            review_log_path,
        );
        assert!(
            review_dispatch_block_reason(&reloaded_state, Some(audit_id), &digest)
                .await
                .unwrap()
                .is_none()
        );
        initial_output.audit_summary.blocked = false;
    }

    #[test]
    fn task_contract_blocks_dispatch_when_required_field_is_local_only() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "internal_case_notes".to_string(),
            privagate_core::FieldPolicy {
                field_type: "secret".to_string(),
                mechanism: privagate_core::Mechanism::LocalOnly,
                preserve_equality: false,
                required_for_task: false,
                bucket_size: None,
                address_level: None,
            },
        );

        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "internal_case_note_triage".to_string(),
            privagate_core::TaskContract {
                required_fields: vec!["internal_case_notes".to_string()],
                allowed_adapter_classes: Vec::new(),
                promotion_utility: privagate_core::PromotionUtilityProfile::default(),
            },
        );

        let state = test_state_with_policy(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "internal_case_note_triage".to_string(),
                key_domain: "local/test".to_string(),
                fields,
                task_contracts,
                constraints: ConstraintPolicy::default(),
                statistics: Vec::new(),
            },
        );

        let assessment = assess_task_contract(&state.policy, "internal_case_note_triage");
        assert!(!assessment.dispatch_allowed);
        let reason =
            task_contract_dispatch_block_reason(&state.policy, "internal_case_note_triage")
                .expect("local_only field should block dispatch");
        assert!(reason.contains("local_only"));
        assert!(reason.contains("internal_case_notes"));
    }

    #[test]
    fn requested_task_profile_overrides_default_profile_for_projection() {
        let base_policy = Policy {
            policy_version: "test".to_string(),
            task_profile: "default_profile".to_string(),
            key_domain: "local/test".to_string(),
            fields: BTreeMap::new(),
            task_contracts: BTreeMap::new(),
            constraints: ConstraintPolicy::default(),
            statistics: Vec::new(),
        };

        let effective_policy = project_policy(&base_policy, Some("override_profile"));
        assert_eq!(effective_policy.task_profile, "override_profile");

        let unchanged_policy = project_policy(&base_policy, Some("   "));
        assert_eq!(unchanged_policy.task_profile, "default_profile");
    }

    #[tokio::test]
    async fn policy_block_sets_projection_blocked_without_manual_review() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "internal_case_notes".to_string(),
            privagate_core::FieldPolicy {
                field_type: "secret".to_string(),
                mechanism: privagate_core::Mechanism::LocalOnly,
                preserve_equality: false,
                required_for_task: false,
                bucket_size: None,
                address_level: None,
            },
        );
        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "internal_case_note_triage".to_string(),
            privagate_core::TaskContract {
                required_fields: vec!["internal_case_notes".to_string()],
                allowed_adapter_classes: Vec::new(),
                promotion_utility: privagate_core::PromotionUtilityProfile::default(),
            },
        );
        let state = test_state_with_policy(
            ReviewMode::Manual,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "internal_case_note_triage".to_string(),
                key_domain: "local/test".to_string(),
                fields,
                task_contracts,
                constraints: ConstraintPolicy::default(),
                statistics: Vec::new(),
            },
        );
        let input = GatewayInput {
            content_type: privagate_core::transform::ContentType::Json,
            payload: serde_json::json!({
                "internal_case_notes": "synthetic-escalation-note"
            }),
        };
        let mut output = process_document(input, &state.policy, state.hmac_key.as_ref()).unwrap();
        let assessment = assess_task_contract(&state.policy, &output.utility_report.task_profile);
        let manual_review = apply_projection_dispatch_controls(&state, &mut output, &assessment)
            .await
            .unwrap();

        assert!(manual_review.is_none());
        assert!(output.audit_summary.blocked);
    }

    #[test]
    fn rag_chunk_request_accepts_optional_task_profile() {
        let request: RagChunkProjectRequest = serde_json::from_value(serde_json::json!({
            "task_profile": "rag_privacy_projection",
            "chunks": [
                {
                    "chunk_id": "chunk-1",
                    "source_uri": "local://doc-1",
                    "content_type": "text",
                    "payload": "synthetic content"
                }
            ]
        }))
        .unwrap();

        assert_eq!(
            request.task_profile.as_deref(),
            Some("rag_privacy_projection")
        );
        assert_eq!(request.chunks.len(), 1);
    }

    #[test]
    fn adapter_capability_blocks_disallowed_adapter_class() {
        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "strict_local_summary".to_string(),
            privagate_core::TaskContract {
                required_fields: vec!["severity".to_string()],
                allowed_adapter_classes: vec![privagate_core::AdapterClass::LocalPrivate],
                promotion_utility: privagate_core::PromotionUtilityProfile::default(),
            },
        );
        let state = test_state_with_policy(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "strict_local_summary".to_string(),
                key_domain: "local/test".to_string(),
                fields: BTreeMap::new(),
                task_contracts,
                constraints: ConstraintPolicy::default(),
                statistics: Vec::new(),
            },
        );

        let capabilities = state.model_adapter.capabilities();
        let reason =
            adapter_capability_block_reason(&state.policy, "strict_local_summary", &capabilities)
                .expect("reserved adapter should be blocked");
        assert!(reason.contains("local_private"));
        assert!(reason.contains("reserved"));
    }

    #[tokio::test]
    async fn route_plan_validation_respects_manual_review_gate() {
        let review_log_path = unique_test_file("route-plan-review");
        let state = test_state_with_policy_and_review_log(
            ReviewMode::Manual,
            base_policy("manual_review_test"),
            review_log_path,
        );
        let input = GatewayInput {
            content_type: privagate_core::transform::ContentType::Text,
            payload: serde_json::json!("synthetic route-plan input"),
        };
        let mut output = process_document(input, &state.policy, state.hmac_key.as_ref()).unwrap();
        create_manual_review_if_required(&state, &mut output)
            .await
            .unwrap()
            .expect("pending review should exist");

        let request = RoutePlanValidationRequest {
            route_id: None,
            aggregation_strategy: Some("merge_stage_outputs".to_string()),
            residual_risk_notes: vec!["synthetic test route".to_string()],
            stages: vec![RouteStageRequest {
                stage_id: "stage-1".to_string(),
                provider: "dry-run".to_string(),
                task_profile: "manual_review_test".to_string(),
                adapter_class: privagate_core::AdapterClass::LocalPrivate,
                audit_id: Some(output.audit_summary.audit_id),
                shard_group: None,
                shard_id: None,
                external_view: output.external_view.clone(),
            }],
        };

        let Json(blocked_response) =
            validate_route_plan(State(state.clone()), Json(request.clone()))
                .await
                .unwrap();
        assert!(!blocked_response.dispatch_allowed);
        assert_eq!(blocked_response.stages.len(), 1);
        assert!(blocked_response.stages[0].blocked_by_review);
        assert_eq!(
            blocked_response.stages[0].external_view_digest,
            output.audit_summary.external_view_digest
        );

        set_manual_review_decision(
            &state,
            ReviewDecisionRequest {
                audit_id: output.audit_summary.audit_id,
                reviewer: "route-plan-reviewer".to_string(),
                reason: Some("approved projected route stage".to_string()),
            },
            ReviewStatus::Approved,
        )
        .await
        .unwrap();

        let Json(approved_response) = validate_route_plan(State(state), Json(request))
            .await
            .unwrap();
        assert!(approved_response.dispatch_allowed);
        assert!(!approved_response.stages[0].blocked_by_review);
        assert!(!approved_response.stages[0].blocked_by_policy);
    }

    #[tokio::test]
    async fn route_plan_validation_blocks_disallowed_adapter_class() {
        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "strict_local_summary".to_string(),
            privagate_core::TaskContract {
                required_fields: vec!["severity".to_string()],
                allowed_adapter_classes: vec![privagate_core::AdapterClass::LocalPrivate],
                promotion_utility: privagate_core::PromotionUtilityProfile::default(),
            },
        );
        let state = test_state_with_policy(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "strict_local_summary".to_string(),
                key_domain: "local/test".to_string(),
                fields: BTreeMap::new(),
                task_contracts,
                constraints: ConstraintPolicy::default(),
                statistics: Vec::new(),
            },
        );

        let request = RoutePlanValidationRequest {
            route_id: Some("synthetic-route".to_string()),
            aggregation_strategy: Some("single_stage".to_string()),
            residual_risk_notes: Vec::new(),
            stages: vec![RouteStageRequest {
                stage_id: "stage-1".to_string(),
                provider: "reserved-adapter".to_string(),
                task_profile: "strict_local_summary".to_string(),
                adapter_class: privagate_core::AdapterClass::Reserved,
                audit_id: None,
                shard_group: None,
                shard_id: None,
                external_view: privagate_core::ExternalView {
                    content_type: privagate_core::transform::ContentType::Json,
                    payload: serde_json::json!({
                        "severity": "high"
                    }),
                },
            }],
        };

        let Json(response) = validate_route_plan(State(state), Json(request))
            .await
            .unwrap();
        assert!(!response.dispatch_allowed);
        assert!(response.stages[0].blocked_by_policy);
        assert!(response.stages[0]
            .issues
            .iter()
            .any(|issue| issue.contains("local_private")));
    }

    #[tokio::test]
    async fn route_plan_execution_dispatches_dry_run_stages() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("multi_stage_summary"),
            Arc::new(DryRunAdapter),
        );
        let request = RoutePlanExecutionRequest {
            route_plan: RoutePlanValidationRequest {
                route_id: Some("route-exec-1".to_string()),
                aggregation_strategy: Some("merge_stage_outputs".to_string()),
                residual_risk_notes: vec!["synthetic execution test".to_string()],
                stages: vec![
                    RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "multi_stage_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: None,
                        shard_id: None,
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high",
                                "role": "guarantor"
                            }),
                        },
                    },
                    RouteStageRequest {
                        stage_id: "stage-2".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "multi_stage_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: None,
                        shard_id: None,
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Text,
                            payload: serde_json::json!("synthetic projected route stage"),
                        },
                    },
                ],
            },
            stop_on_block: true,
        };

        let Json(response) = execute_route_plan(State(state), Json(request))
            .await
            .unwrap();
        assert!(response.dispatch_allowed);
        assert!(!response.halted);
        assert_eq!(response.executed_stage_count, 2);
        assert_eq!(response.dispatched_stage_count, 2);
        assert!(response.all_stages_dispatched);
        assert_eq!(
            response.runtime_adapter_capabilities.adapter_class,
            privagate_core::AdapterClass::LocalPrivate
        );
        assert!(response.stages.iter().all(|stage| stage.executed));
        assert!(response
            .stages
            .iter()
            .all(|stage| stage.runtime_issues.is_empty()));
        assert!(response.stages.iter().all(|stage| stage
            .dispatch_response
            .as_ref()
            .and_then(|dispatch| dispatch.output.as_ref())
            .is_some_and(|output| output.contains("\"adapter\":\"dry_run\""))));
    }

    #[tokio::test]
    async fn route_plan_execution_halts_on_runtime_adapter_mismatch() {
        let state = test_state_with_policy(ReviewMode::Off, base_policy("multi_stage_summary"));
        let request = RoutePlanExecutionRequest {
            route_plan: RoutePlanValidationRequest {
                route_id: Some("route-exec-mismatch".to_string()),
                aggregation_strategy: Some("merge_stage_outputs".to_string()),
                residual_risk_notes: Vec::new(),
                stages: vec![
                    RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "declared-local-private".to_string(),
                        task_profile: "multi_stage_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: None,
                        shard_id: None,
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "medium"
                            }),
                        },
                    },
                    RouteStageRequest {
                        stage_id: "stage-2".to_string(),
                        provider: "declared-local-private".to_string(),
                        task_profile: "multi_stage_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: None,
                        shard_id: None,
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "low"
                            }),
                        },
                    },
                ],
            },
            stop_on_block: true,
        };

        let Json(response) = execute_route_plan(State(state), Json(request))
            .await
            .unwrap();
        assert!(response.dispatch_allowed);
        assert!(response.halted);
        assert_eq!(response.executed_stage_count, 0);
        assert_eq!(response.dispatched_stage_count, 0);
        assert!(!response.all_stages_dispatched);
        assert!(response.stages[0]
            .runtime_issues
            .iter()
            .any(|issue| issue.contains("configured adapter_class=reserved")));
        assert_eq!(
            response.stages[1].skipped_reason.as_deref(),
            Some("route execution stopped after an earlier stage was blocked or not dispatched")
        );
        assert!(response.stages[1].dispatch_response.is_none());
    }

    #[tokio::test]
    async fn shard_plan_validation_detects_missing_groups_and_duplicate_shards() {
        let state = test_state_with_policy(ReviewMode::Off, base_policy("sharded_summary"));
        let request = ShardPlanValidationRequest {
            route_plan: RoutePlanValidationRequest {
                route_id: Some("shard-validate-1".to_string()),
                aggregation_strategy: Some("local_merge".to_string()),
                residual_risk_notes: vec!["synthetic shard validation".to_string()],
                stages: vec![
                    RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run-a".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("shard-a".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    },
                    RouteStageRequest {
                        stage_id: "stage-2".to_string(),
                        provider: "dry-run-a".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("shard-a".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Text,
                            payload: serde_json::json!("synthetic shard text"),
                        },
                    },
                ],
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: true,
                strategy: LocalAggregationStrategy::DigestOnly,
                promotion: AggregationPromotionRules::default(),
                expected_groups: vec![
                    ShardGroupExpectation {
                        group_id: "claims".to_string(),
                        min_shards: 2,
                        strategy: None,
                        promotion: None,
                    },
                    ShardGroupExpectation {
                        group_id: "risk".to_string(),
                        min_shards: 1,
                        strategy: None,
                        promotion: None,
                    },
                ],
            },
        };

        let Json(response) = validate_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        assert!(
            !response
                .local_aggregation_summary
                .ready_for_local_aggregation
        );
        assert!(response
            .local_aggregation_summary
            .issues
            .iter()
            .any(|issue| issue.contains("expected shard_group=risk is missing")));
        assert!(response
            .local_aggregation_summary
            .issues
            .iter()
            .any(|issue| issue.contains("duplicate shard_id")));
        assert!(response
            .local_aggregation_summary
            .issues
            .iter()
            .any(|issue| issue.contains("requires distinct providers")));
    }

    #[tokio::test]
    async fn shard_plan_execution_emits_local_aggregation_digest_for_complete_group() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("sharded_summary"),
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-exec-1".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: vec!["synthetic shard execution".to_string()],
                    stages: vec![
                        RouteStageRequest {
                            stage_id: "stage-1".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("shard-a".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "high",
                                    "role": "guarantor"
                                }),
                            },
                        },
                        RouteStageRequest {
                            stage_id: "stage-2".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("shard-b".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Text,
                                payload: serde_json::json!("synthetic projected shard"),
                            },
                        },
                    ],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::CollectOutputs,
                promotion: AggregationPromotionRules::default(),
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 2,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        assert!(
            response
                .local_aggregation_summary
                .ready_for_local_aggregation
        );
        assert_eq!(response.local_aggregation_summary.groups.len(), 1);
        let group = &response.local_aggregation_summary.groups[0];
        assert_eq!(group.group_id, "claims");
        assert_eq!(
            group.aggregation_strategy,
            LocalAggregationStrategy::CollectOutputs
        );
        assert_eq!(group.dispatched_stage_count, 2);
        assert!(group.all_stage_outputs_bound);
        assert_eq!(group.output_digests.len(), 2);
        assert!(group.local_aggregation_digest.is_some());
        assert!(group.local_only_output_digest.is_some());
        assert!(group.promotion.is_none());
        let local_only_output = group
            .local_only_output
            .as_ref()
            .expect("collect_outputs should materialize a local-only output");
        let outputs = local_only_output
            .as_array()
            .expect("collect_outputs should build an array");
        assert_eq!(outputs.len(), 2);
        assert!(outputs.iter().all(|item| item.get("output").is_some()));
    }

    #[tokio::test]
    async fn shard_plan_execution_honors_group_strategy_override() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("sharded_summary"),
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-exec-override".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![
                        RouteStageRequest {
                            stage_id: "stage-1".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("alpha".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "high"
                                }),
                            },
                        },
                        RouteStageRequest {
                            stage_id: "stage-2".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("beta".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "medium"
                                }),
                            },
                        },
                    ],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::DigestOnly,
                promotion: AggregationPromotionRules::default(),
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 2,
                    strategy: Some(LocalAggregationStrategy::JsonObjectByShard),
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let group = &response.local_aggregation_summary.groups[0];
        assert_eq!(
            group.aggregation_strategy,
            LocalAggregationStrategy::JsonObjectByShard
        );
        let local_only_output = group
            .local_only_output
            .as_ref()
            .expect("group strategy override should produce an object output");
        let object = local_only_output
            .as_object()
            .expect("json_object_by_shard should build an object");
        assert!(object.contains_key("alpha"));
        assert!(object.contains_key("beta"));
    }

    #[tokio::test]
    async fn shard_plan_execution_promotes_local_aggregation_to_external_view_candidate() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("follow_up_summary"),
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-1".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![
                        RouteStageRequest {
                            stage_id: "stage-1".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("alpha".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "high"
                                }),
                            },
                        },
                        RouteStageRequest {
                            stage_id: "stage-2".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("beta".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "medium"
                                }),
                            },
                        },
                    ],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 2,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(promotion.promotion_allowed);
        assert!(promotion.ready_for_follow_up_route);
        assert!(promotion.issues.is_empty());
        assert_eq!(
            promotion.candidate_task_profile.as_deref(),
            Some("follow_up_summary")
        );
        assert_eq!(
            promotion.candidate_content_type,
            Some(privagate_core::transform::ContentType::Json)
        );
        assert!(promotion.external_view_candidate_digest.is_some());
        let candidate = promotion
            .external_view_candidate
            .as_ref()
            .expect("candidate external view should exist");
        assert_eq!(
            candidate.content_type,
            privagate_core::transform::ContentType::Json
        );
        assert!(candidate.payload.get("alpha").is_some());
        assert!(candidate.payload.get("beta").is_some());
    }

    #[tokio::test]
    async fn shard_plan_execution_blocks_promotion_when_follow_up_task_contract_is_incompatible() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "internal_case_notes".to_string(),
            privagate_core::FieldPolicy {
                field_type: "secret".to_string(),
                mechanism: privagate_core::Mechanism::LocalOnly,
                preserve_equality: false,
                required_for_task: false,
                bucket_size: None,
                address_level: None,
            },
        );
        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "restricted_follow_up".to_string(),
            privagate_core::TaskContract {
                required_fields: vec!["internal_case_notes".to_string()],
                allowed_adapter_classes: Vec::new(),
                promotion_utility: privagate_core::PromotionUtilityProfile::default(),
            },
        );
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "sharded_summary".to_string(),
                key_domain: "local/test".to_string(),
                fields,
                task_contracts,
                constraints: ConstraintPolicy::default(),
                statistics: Vec::new(),
            },
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-blocked".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("restricted_follow_up".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: Vec::new(),
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(!promotion.promotion_allowed);
        assert!(!promotion.ready_for_follow_up_route);
        assert!(promotion.external_view_candidate.is_none());
        assert!(promotion
            .task_contract_assessment
            .as_ref()
            .is_some_and(|assessment| !assessment.dispatch_allowed));
        assert!(promotion
            .issues
            .iter()
            .any(|issue| issue.contains("local_only")));
    }

    #[tokio::test]
    async fn shard_plan_execution_candidate_is_not_route_ready_in_manual_review_mode() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Manual,
            base_policy("follow_up_summary"),
            Arc::new(DryRunAdapter),
        );
        let input = GatewayInput {
            content_type: privagate_core::transform::ContentType::Json,
            payload: serde_json::json!({
                "severity": "high"
            }),
        };
        let mut output = process_document(input, &state.policy, state.hmac_key.as_ref()).unwrap();
        create_manual_review_if_required(&state, &mut output)
            .await
            .unwrap()
            .expect("manual review record should exist");
        set_manual_review_decision(
            &state,
            ReviewDecisionRequest {
                audit_id: output.audit_summary.audit_id,
                reviewer: "manual-shard-reviewer".to_string(),
                reason: Some("approved shard stage".to_string()),
            },
            ReviewStatus::Approved,
        )
        .await
        .unwrap();

        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-manual".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: Some(output.audit_summary.audit_id),
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: output.external_view.clone(),
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: Vec::new(),
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(promotion.promotion_allowed);
        assert!(!promotion.ready_for_follow_up_route);
        assert!(promotion.external_view_candidate.is_some());
        assert!(promotion
            .issues
            .iter()
            .any(|issue| issue.contains("manual review mode")));
    }

    #[tokio::test]
    async fn shard_plan_execution_blocks_promotion_when_output_exceeds_item_limit() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("follow_up_summary"),
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-limit".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![
                        RouteStageRequest {
                            stage_id: "stage-1".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("alpha".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "high"
                                }),
                            },
                        },
                        RouteStageRequest {
                            stage_id: "stage-2".to_string(),
                            provider: "dry-run".to_string(),
                            task_profile: "sharded_summary".to_string(),
                            adapter_class: privagate_core::AdapterClass::LocalPrivate,
                            audit_id: None,
                            shard_group: Some("claims".to_string()),
                            shard_id: Some("beta".to_string()),
                            external_view: privagate_core::ExternalView {
                                content_type: privagate_core::transform::ContentType::Json,
                                payload: serde_json::json!({
                                    "severity": "medium"
                                }),
                            },
                        },
                    ],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::CollectOutputs,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: Vec::new(),
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: Some(1),
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 2,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(!promotion.promotion_allowed);
        assert!(promotion.external_view_candidate.is_none());
        assert!(promotion
            .issues
            .iter()
            .any(|issue| issue.contains("max_array_items=1")));
    }

    #[tokio::test]
    async fn bind_shard_promotion_creates_follow_up_binding_when_review_is_off() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("follow_up_summary"),
            Arc::new(DryRunAdapter),
        );
        let execute_request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("bind-promotion-off".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(executed) = execute_shard_plan(State(state.clone()), Json(execute_request))
            .await
            .unwrap();
        let bind_request = ShardPromotionBindingRequest {
            route_plan_execution: executed.route_plan_execution.clone(),
            aggregation_rules: executed.aggregation_rules.clone(),
            group_id: "claims".to_string(),
        };

        let response = bind_local_shard_promotion(&state, &bind_request)
            .await
            .unwrap();
        assert!(response.binding_created);
        assert!(response.ready_for_follow_up_route);
        assert!(response.issues.is_empty());
        assert!(response.manual_review.is_none());
        let binding = response
            .follow_up_binding
            .as_ref()
            .expect("binding should exist");
        assert_eq!(binding.task_profile, "follow_up_summary");
        assert!(!binding.audit_summary.blocked);
        assert!(binding.utility_assessment.verification_passed);
        assert_eq!(
            response.source_local_only_output_digest.as_deref(),
            Some(binding.audit_summary.input_digest.as_str())
        );
        assert_eq!(
            binding.external_view.content_type,
            privagate_core::transform::ContentType::Json
        );
    }

    #[tokio::test]
    async fn bind_shard_promotion_creates_pending_manual_review_binding() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Manual,
            base_policy("follow_up_summary"),
            Arc::new(DryRunAdapter),
        );
        let input = GatewayInput {
            content_type: privagate_core::transform::ContentType::Json,
            payload: serde_json::json!({
                "severity": "high"
            }),
        };
        let mut projected =
            process_document(input, &state.policy, state.hmac_key.as_ref()).unwrap();
        create_manual_review_if_required(&state, &mut projected)
            .await
            .unwrap()
            .expect("stage manual review should be created");
        set_manual_review_decision(
            &state,
            ReviewDecisionRequest {
                audit_id: projected.audit_summary.audit_id,
                reviewer: "stage-reviewer".to_string(),
                reason: Some("approved upstream stage".to_string()),
            },
            ReviewStatus::Approved,
        )
        .await
        .unwrap();

        let execute_request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("bind-promotion-manual".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: Some(projected.audit_summary.audit_id),
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: projected.external_view.clone(),
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(executed) = execute_shard_plan(State(state.clone()), Json(execute_request))
            .await
            .unwrap();
        let bind_request = ShardPromotionBindingRequest {
            route_plan_execution: executed.route_plan_execution.clone(),
            aggregation_rules: executed.aggregation_rules.clone(),
            group_id: "claims".to_string(),
        };

        let response = bind_local_shard_promotion(&state, &bind_request)
            .await
            .unwrap();
        assert!(response.binding_created);
        assert!(!response.ready_for_follow_up_route);
        let review = response
            .manual_review
            .as_ref()
            .expect("manual review should be created for follow-up binding");
        assert_eq!(review.status, ReviewStatus::Pending);
        let binding = response
            .follow_up_binding
            .as_ref()
            .expect("binding should exist");
        assert!(binding.audit_summary.blocked);
        assert!(binding.utility_assessment.verification_passed);
        assert!(review_dispatch_block_reason(
            &state,
            Some(binding.audit_summary.audit_id),
            &binding.audit_summary.external_view_digest,
        )
        .await
        .unwrap()
        .is_some_and(|reason| reason.contains("pending manual review")));
    }

    #[tokio::test]
    async fn bind_shard_promotion_blocks_tampered_execution_evidence() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            base_policy("follow_up_summary"),
            Arc::new(DryRunAdapter),
        );
        let execute_request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("bind-promotion-tampered".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(mut executed) = execute_shard_plan(State(state.clone()), Json(execute_request))
            .await
            .unwrap();
        executed.route_plan_execution.stages[0].dispatch_output_digest =
            Some("sha256:tampered".to_string());
        let bind_request = ShardPromotionBindingRequest {
            route_plan_execution: executed.route_plan_execution,
            aggregation_rules: executed.aggregation_rules,
            group_id: "claims".to_string(),
        };

        let response = bind_local_shard_promotion(&state, &bind_request)
            .await
            .unwrap();
        assert!(!response.binding_created);
        assert!(!response.ready_for_follow_up_route);
        assert!(response.follow_up_binding.is_none());
        assert!(response
            .issues
            .iter()
            .any(|issue| issue.contains("dispatch_output_digest mismatch")));
    }

    #[tokio::test]
    async fn shard_plan_execution_blocks_promotion_when_required_fields_are_missing() {
        let mut fields = BTreeMap::new();
        for field_name in [
            "company_name",
            "contract_id",
            "contract_amount_bucket",
            "severity",
        ] {
            fields.insert(
                field_name.to_string(),
                privagate_core::FieldPolicy {
                    field_type: field_name.to_string(),
                    mechanism: privagate_core::Mechanism::Passthrough,
                    preserve_equality: true,
                    required_for_task: true,
                    bucket_size: None,
                    address_level: None,
                },
            );
        }
        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "contract_risk_review".to_string(),
            privagate_core::TaskContract {
                required_fields: vec![
                    "company_name".to_string(),
                    "contract_id".to_string(),
                    "contract_amount_bucket".to_string(),
                    "severity".to_string(),
                ],
                allowed_adapter_classes: vec![privagate_core::AdapterClass::LocalPrivate],
                promotion_utility: privagate_core::PromotionUtilityProfile::default(),
            },
        );
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "sharded_summary".to_string(),
                key_domain: "local/test".to_string(),
                fields,
                task_contracts,
                constraints: ConstraintPolicy::default(),
                statistics: Vec::new(),
            },
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-required-fields".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("contract_risk_review".to_string()),
                    utility_verification: PromotionUtilityVerificationRules::default(),
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(!promotion.promotion_allowed);
        let utility = promotion
            .utility_assessment
            .as_ref()
            .expect("utility assessment should be present");
        assert!(!utility.verification_passed);
        assert_eq!(utility.required_field_presence.required, 4);
        assert!(utility
            .missing_required_fields
            .iter()
            .any(|field| field == "company_name"));
        assert!(promotion
            .issues
            .iter()
            .any(|issue| issue.contains("missing required field=company_name")));
    }

    #[tokio::test]
    async fn shard_plan_execution_blocks_promotion_when_constraint_verification_fails() {
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "sharded_summary".to_string(),
                key_domain: "local/test".to_string(),
                fields: BTreeMap::new(),
                task_contracts: BTreeMap::new(),
                constraints: ConstraintPolicy {
                    preserve_relations: true,
                    preserve_time_order: false,
                    preserve_foreign_keys: false,
                    foreign_keys: Vec::new(),
                    time_orders: Vec::new(),
                    relations: Vec::new(),
                },
                statistics: Vec::new(),
            },
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-constraint-check".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules {
                        require_required_fields: false,
                        verify_constraint_results: true,
                    },
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(!promotion.promotion_allowed);
        let utility = promotion
            .utility_assessment
            .as_ref()
            .expect("utility assessment should be present");
        assert!(!utility.verification_passed);
        assert!(utility
            .constraint_results
            .iter()
            .any(|result| result.check == "relation_preservation" && !result.passed));
        assert!(promotion
            .issues
            .iter()
            .any(|issue| issue.contains("follow-up utility check failed relation_preservation")));
    }

    #[tokio::test]
    async fn shard_plan_execution_uses_task_profile_specific_promotion_utility_checks() {
        let mut task_contracts = BTreeMap::new();
        task_contracts.insert(
            "follow_up_summary".to_string(),
            privagate_core::TaskContract {
                required_fields: Vec::new(),
                allowed_adapter_classes: vec![privagate_core::AdapterClass::LocalPrivate],
                promotion_utility: privagate_core::PromotionUtilityProfile {
                    require_required_fields: false,
                    required_constraint_checks: vec![
                        privagate_core::UtilityConstraintCheck::RelationPreservation,
                    ],
                },
            },
        );
        let state = test_state_with_policy_and_adapter(
            ReviewMode::Off,
            Policy {
                policy_version: "test".to_string(),
                task_profile: "sharded_summary".to_string(),
                key_domain: "local/test".to_string(),
                fields: BTreeMap::new(),
                task_contracts,
                constraints: ConstraintPolicy {
                    preserve_relations: false,
                    preserve_time_order: false,
                    preserve_foreign_keys: false,
                    foreign_keys: Vec::new(),
                    time_orders: Vec::new(),
                    relations: Vec::new(),
                },
                statistics: Vec::new(),
            },
            Arc::new(DryRunAdapter),
        );
        let request = ShardPlanExecutionRequest {
            route_plan_execution: RoutePlanExecutionRequest {
                route_plan: RoutePlanValidationRequest {
                    route_id: Some("shard-promote-task-profile-utility".to_string()),
                    aggregation_strategy: Some("local_merge".to_string()),
                    residual_risk_notes: Vec::new(),
                    stages: vec![RouteStageRequest {
                        stage_id: "stage-1".to_string(),
                        provider: "dry-run".to_string(),
                        task_profile: "sharded_summary".to_string(),
                        adapter_class: privagate_core::AdapterClass::LocalPrivate,
                        audit_id: None,
                        shard_group: Some("claims".to_string()),
                        shard_id: Some("alpha".to_string()),
                        external_view: privagate_core::ExternalView {
                            content_type: privagate_core::transform::ContentType::Json,
                            payload: serde_json::json!({
                                "severity": "high"
                            }),
                        },
                    }],
                },
                stop_on_block: true,
            },
            aggregation_rules: ShardAggregationRules {
                require_shard_metadata: true,
                require_distinct_providers: false,
                strategy: LocalAggregationStrategy::JsonObjectByShard,
                promotion: AggregationPromotionRules {
                    mode: AggregationPromotionMode::ExternalViewCandidate,
                    candidate_task_profile: Some("follow_up_summary".to_string()),
                    utility_verification: PromotionUtilityVerificationRules {
                        require_required_fields: false,
                        verify_constraint_results: false,
                    },
                    allowed_content_types: vec![privagate_core::transform::ContentType::Json],
                    max_serialized_bytes: None,
                    max_text_chars: None,
                    max_array_items: None,
                    max_object_keys: None,
                },
                expected_groups: vec![ShardGroupExpectation {
                    group_id: "claims".to_string(),
                    min_shards: 1,
                    strategy: None,
                    promotion: None,
                }],
            },
        };

        let Json(response) = execute_shard_plan(State(state), Json(request))
            .await
            .unwrap();
        let promotion = response.local_aggregation_summary.groups[0]
            .promotion
            .as_ref()
            .expect("promotion summary should be present");
        assert!(!promotion.promotion_allowed);
        let utility = promotion
            .utility_assessment
            .as_ref()
            .expect("utility assessment should be present");
        assert!(!utility.verification_passed);
        assert!(utility
            .constraint_results
            .iter()
            .any(|result| result.check == "relation_preservation" && !result.passed));
        assert!(promotion
            .issues
            .iter()
            .any(|issue| issue.contains("policy does not enable preserve_relations")));
    }
}
