use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyProofReport {
    pub report_type: String,
    pub report_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub policy_version: String,
    pub input_digest: String,
    pub external_view_digest: String,
    pub mechanisms: Vec<MechanismEvidence>,
    pub privacy_budget: PrivacyBudget,
    pub declared_leakage: Vec<String>,
    pub residual_risks: Vec<String>,
    pub verification_results: Vec<VerificationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismEvidence {
    pub field_name: String,
    pub field_type: String,
    pub mechanism: String,
    pub key_domain: Option<String>,
    pub token_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrivacyBudget {
    pub epsilon: Option<f64>,
    pub delta: Option<f64>,
    pub consumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub check: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityProofReport {
    pub report_type: String,
    pub report_id: Uuid,
    pub task_profile: String,
    pub external_view_digest: String,
    pub entity_preservation: PreservationMetric,
    pub relation_preservation: PreservationMetric,
    pub constraint_results: Vec<VerificationResult>,
    pub statistical_error_bounds: Vec<String>,
    pub task_loss_bounds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationMetric {
    pub required: usize,
    pub preserved: usize,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub audit_id: Uuid,
    pub input_digest: String,
    pub external_view_digest: String,
    pub policy_version: String,
    pub blocked: bool,
}
