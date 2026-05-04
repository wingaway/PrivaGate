mod model_adapter;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use proofgate_core::{
    compute_statistics, process_document, GatewayInput, GatewayOutput, Policy, StatisticOutput,
};
use serde::{Deserialize, Serialize};
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
    DisabledModelAdapter, ModelAdapter, ModelDispatchRequest, ModelDispatchResponse,
};

const SERVICE_NAME: &str = "proofgate-gateway";
const CREATE_POSTGRES_AUDIT_TABLE_SQL: &str = "create table if not exists proofgate_audit_log (
    id bigserial primary key,
    created_at timestamptz not null default now(),
    record jsonb not null
);";
const INSERT_POSTGRES_AUDIT_ROW_SQL: &str = "insert into proofgate_audit_log (record) values ($1)";

#[derive(Clone)]
struct AppState {
    policy: Arc<Policy>,
    hmac_key: Arc<Vec<u8>>,
    mapping_log: Arc<Mutex<MappingLog>>,
    audit_sink: Arc<Mutex<AuditSink>>,
    model_adapter: Arc<dyn ModelAdapter>,
}

struct MappingLog {
    path: PathBuf,
}

enum AuditSink {
    Jsonl { path: PathBuf },
    Postgres { connection_string: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let state = load_state()?;
    let bind = env::var("PROOFGATE_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = bind.parse().context("invalid PROOFGATE_BIND")?;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/project", post(project))
        .route("/v1/statistics", post(statistics))
        .route("/v1/rag/project-chunks", post(project_rag_chunks))
        .route("/v1/tool/inspect", post(inspect_tool_io))
        .route("/v1/session/risk", post(session_risk))
        .route("/v1/inspect-output", post(inspect_output))
        .route("/v1/restore-output", post(restore_output))
        .route("/v1/model-dispatch", post(model_dispatch))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "ProofGate gateway listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn project_rag_chunks(
    State(state): State<AppState>,
    Json(input): Json<RagChunkProjectRequest>,
) -> Result<Json<RagChunkProjectResponse>, ApiError> {
    let mut chunks = Vec::new();
    for chunk in input.chunks {
        let gateway_input = GatewayInput {
            content_type: chunk.content_type,
            payload: chunk.payload,
        };
        let output = process_document(gateway_input, &state.policy, &state.hmac_key)?;
        persist_local_mappings(&state, &output)?;
        persist_audit_summary(&state, &output).await?;
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
    let policy_path = env::var("PROOFGATE_POLICY_PATH")
        .context("PROOFGATE_POLICY_PATH must point to a local policy JSON file")?;
    let hmac_key = env::var("PROOFGATE_HMAC_KEY")
        .context("PROOFGATE_HMAC_KEY must be set from local secret storage")?;
    let mapping_log_path = env::var("PROOFGATE_MAPPING_LOG")
        .unwrap_or_else(|_| "data/local-mappings.jsonl".to_string());
    let audit_sink = match env::var("PROOFGATE_AUDIT_POSTGRES_URL") {
        Ok(connection_string) if !connection_string.trim().is_empty() => {
            AuditSink::Postgres { connection_string }
        }
        _ => AuditSink::Jsonl {
            path: PathBuf::from(
                env::var("PROOFGATE_AUDIT_LOG").unwrap_or_else(|_| "data/audit.jsonl".to_string()),
            ),
        },
    };
    let policy_json = fs::read_to_string(&policy_path)
        .with_context(|| format!("failed to read policy file: {policy_path}"))?;
    let policy: Policy = serde_json::from_str(&policy_json)
        .with_context(|| format!("failed to parse policy file: {policy_path}"))?;

    Ok(AppState {
        policy: Arc::new(policy),
        hmac_key: Arc::new(hmac_key.into_bytes()),
        mapping_log: Arc::new(Mutex::new(MappingLog {
            path: PathBuf::from(mapping_log_path),
        })),
        audit_sink: Arc::new(Mutex::new(audit_sink)),
        model_adapter: Arc::new(DisabledModelAdapter),
    })
}

async fn healthz() -> Json<Health> {
    Json(Health {
        status: "ok",
        service: SERVICE_NAME,
    })
}

async fn project(
    State(state): State<AppState>,
    Json(input): Json<GatewayInput>,
) -> Result<Json<GatewayOutput>, ApiError> {
    let output = process_document(input, &state.policy, &state.hmac_key)?;
    persist_local_mappings(&state, &output)?;
    persist_audit_summary(&state, &output).await?;
    Ok(Json(output))
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
) -> Json<ModelDispatchResponse> {
    Json(state.model_adapter.dispatch(input))
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

async fn persist_audit_summary(state: &AppState, output: &GatewayOutput) -> Result<()> {
    let row = serde_json::json!({
        "audit_summary": &output.audit_summary,
        "privacy_report_id": output.privacy_report.report_id,
        "utility_report_id": output.utility_report.report_id,
        "privacy_verification_results": &output.privacy_report.verification_results,
        "utility_constraint_results": &output.utility_report.constraint_results,
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
    chunks: Vec<RagChunkInput>,
}

#[derive(Deserialize)]
struct RagChunkInput {
    chunk_id: String,
    source_uri: String,
    content_type: proofgate_core::transform::ContentType,
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
    external_view: proofgate_core::ExternalView,
    privacy_passed: bool,
    utility_passed: bool,
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
            "error": "proofgate_error",
            "message": self.0.to_string()
        });
        (StatusCode::BAD_REQUEST, Json(body)).into_response()
    }
}
