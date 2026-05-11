use privagate_core::{AdapterClass, ExternalView};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDispatchRequest {
    pub provider: String,
    pub task_profile: String,
    pub audit_id: Option<Uuid>,
    pub external_view: ExternalView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDispatchResponse {
    pub provider: String,
    pub dispatched: bool,
    pub status: String,
    pub output: Option<String>,
    pub audit_id: Option<Uuid>,
    pub external_view_digest: Option<String>,
    pub blocked_by_review: bool,
    pub blocked_by_policy: bool,
    pub adapter_capabilities: AdapterCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    pub adapter_class: AdapterClass,
    pub accepts_external_view_only: bool,
    pub supports_digest_binding: bool,
    pub supports_manual_review_gate: bool,
}

pub trait ModelAdapter: Send + Sync {
    fn capabilities(&self) -> AdapterCapabilities;
    fn dispatch(&self, request: ModelDispatchRequest) -> ModelDispatchResponse;
}

#[derive(Debug, Default)]
pub struct DisabledModelAdapter;

impl ModelAdapter for DisabledModelAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            adapter_class: AdapterClass::Reserved,
            accepts_external_view_only: true,
            supports_digest_binding: true,
            supports_manual_review_gate: true,
        }
    }

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
            blocked_by_policy: false,
            adapter_capabilities: self.capabilities(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DryRunAdapter;

impl ModelAdapter for DryRunAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            adapter_class: AdapterClass::LocalPrivate,
            accepts_external_view_only: true,
            supports_digest_binding: true,
            supports_manual_review_gate: true,
        }
    }

    fn dispatch(&self, request: ModelDispatchRequest) -> ModelDispatchResponse {
        let summary = serde_json::json!({
            "adapter": "dry_run",
            "provider": request.provider,
            "task_profile": request.task_profile,
            "content_type": &request.external_view.content_type,
            "payload_summary": summarize_payload(&request.external_view.payload),
            "audit_id": request.audit_id,
        });

        ModelDispatchResponse {
            provider: request.provider,
            dispatched: true,
            status: "dry_run adapter accepted projected request".to_string(),
            output: Some(summary.to_string()),
            audit_id: request.audit_id,
            external_view_digest: None,
            blocked_by_review: false,
            blocked_by_policy: false,
            adapter_capabilities: self.capabilities(),
        }
    }
}

fn summarize_payload(payload: &Value) -> Value {
    match payload {
        Value::Object(map) => serde_json::json!({
            "shape": "object",
            "field_count": map.len(),
            "field_names": map.keys().collect::<Vec<_>>(),
        }),
        Value::Array(items) => serde_json::json!({
            "shape": "array",
            "item_count": items.len(),
        }),
        Value::String(text) => serde_json::json!({
            "shape": "string",
            "char_count": text.chars().count(),
        }),
        Value::Number(number) => serde_json::json!({
            "shape": "number",
            "value": number,
        }),
        Value::Bool(flag) => serde_json::json!({
            "shape": "bool",
            "value": flag,
        }),
        Value::Null => serde_json::json!({
            "shape": "null",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privagate_core::transform::ContentType;

    #[test]
    fn dry_run_adapter_returns_projected_summary() {
        let adapter = DryRunAdapter;
        let response = adapter.dispatch(ModelDispatchRequest {
            provider: "dry-run".to_string(),
            task_profile: "contract_risk_review".to_string(),
            audit_id: Some(Uuid::nil()),
            external_view: ExternalView {
                content_type: ContentType::Json,
                payload: serde_json::json!({
                    "role": "guarantor",
                    "severity": "high"
                }),
            },
        });

        assert!(response.dispatched);
        assert_eq!(
            response.adapter_capabilities.adapter_class,
            AdapterClass::LocalPrivate
        );
        let output = response.output.expect("dry-run output should exist");
        assert!(output.contains("\"adapter\":\"dry_run\""));
        assert!(output.contains("\"field_count\":2"));
    }
}
