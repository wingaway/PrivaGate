use proofgate_core::ExternalView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDispatchRequest {
    pub provider: String,
    pub task_profile: String,
    pub external_view: ExternalView,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDispatchResponse {
    pub provider: String,
    pub dispatched: bool,
    pub status: String,
    pub output: Option<String>,
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
        }
    }
}
