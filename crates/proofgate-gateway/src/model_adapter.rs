use proofgate_core::ExternalView;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDispatchRequest {
    pub provider: String,
    pub task_profile: String,
    pub audit_id: Option<Uuid>,
    pub external_view: ExternalView,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDispatchResponse {
    pub provider: String,
    pub dispatched: bool,
    pub status: String,
    pub output: Option<String>,
    pub audit_id: Option<Uuid>,
    pub external_view_digest: Option<String>,
    pub blocked_by_review: bool,
}

pub trait ModelAdapter: Send + Sync {
    fn dispatch(&self, request: ModelDispatchRequest) -> ModelDispatchResponse;
}

#[derive(Debug, Default)]
pub struct DisabledModelAdapter;

impl ModelAdapter for DisabledModelAdapter {
    fn dispatch(&self, request: ModelDispatchRequest) -> ModelDispatchResponse {
        let _ = request.external_view;
        ModelDispatchResponse {
            provider: request.provider,
            dispatched: false,
            status: format!(
                "external model adapter is reserved but not configured for task_profile={}",
                request.task_profile
            ),
            output: None,
            audit_id: request.audit_id,
            external_view_digest: None,
            blocked_by_review: false,
        }
    }
}
