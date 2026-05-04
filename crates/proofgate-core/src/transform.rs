use crate::digest::sha256_json;
use crate::policy::{AddressLevel, Mechanism, Policy};
use crate::report::{
    AuditSummary, MechanismEvidence, PrivacyBudget, PrivacyProofReport, UtilityProofReport,
    VerificationResult,
};
use crate::token::hmac_token;
use crate::verify::{
    foreign_key_result, no_direct_identifier_residue, preservation, relation_result,
    statistical_error_bound, time_order_result,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInput {
    pub content_type: ContentType,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    Json,
    CsvRows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalView {
    pub content_type: ContentType,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayOutput {
    pub external_view: ExternalView,
    pub privacy_report: PrivacyProofReport,
    pub utility_report: UtilityProofReport,
    pub audit_summary: AuditSummary,
    #[serde(skip)]
    pub local_mappings: Vec<LocalMappingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMappingEntry {
    pub field_name: String,
    pub field_type: String,
    pub token: String,
    pub original_value: String,
}

#[derive(Debug, Clone)]
struct TransformState {
    original_direct_identifiers: Vec<String>,
    mechanism_evidence: Vec<MechanismEvidence>,
    local_mappings: Vec<LocalMappingEntry>,
    token_by_original: BTreeMap<String, String>,
    relative_time_base: BTreeMap<String, DateTime<Utc>>,
    required_fields: usize,
    preserved_fields: usize,
}

pub fn process_document(
    input: GatewayInput,
    policy: &Policy,
    key: &[u8],
) -> Result<GatewayOutput, serde_json::Error> {
    let input_digest = sha256_json(&input)?;
    let mut state = TransformState {
        original_direct_identifiers: Vec::new(),
        mechanism_evidence: Vec::new(),
        local_mappings: Vec::new(),
        token_by_original: BTreeMap::new(),
        relative_time_base: BTreeMap::new(),
        required_fields: 0,
        preserved_fields: 0,
    };

    let payload = transform_payload(
        input.content_type.clone(),
        input.payload,
        policy,
        key,
        &mut state,
    );
    let external_view = ExternalView {
        content_type: input.content_type,
        payload,
    };
    let external_view_digest = sha256_json(&external_view)?;
    let external_json = serde_json::to_string(&external_view)?;
    let residue_check =
        no_direct_identifier_residue(&state.original_direct_identifiers, &external_json);

    let privacy_report = PrivacyProofReport {
        report_type: "privacy_proof".to_string(),
        report_id: Uuid::new_v4(),
        created_at: Utc::now(),
        policy_version: policy.policy_version.clone(),
        input_digest: input_digest.clone(),
        external_view_digest: external_view_digest.clone(),
        mechanisms: state.mechanism_evidence,
        privacy_budget: privacy_budget(policy),
        declared_leakage: declared_leakage(policy),
        residual_risks: residual_risks(policy),
        verification_results: vec![residue_check],
    };

    let constraint_results = constraint_results(&external_view.payload, policy);
    let relation_required = policy.constraints.relations.len();
    let relation_preserved = constraint_results
        .iter()
        .filter(|result| result.check == "relation_preservation" && result.passed)
        .count();

    let utility_report = UtilityProofReport {
        report_type: "utility_proof".to_string(),
        report_id: Uuid::new_v4(),
        task_profile: policy.task_profile.clone(),
        external_view_digest: external_view_digest.clone(),
        entity_preservation: preservation(state.required_fields, state.preserved_fields),
        relation_preservation: preservation(relation_required, relation_preserved),
        constraint_results,
        statistical_error_bounds: policy
            .statistics
            .iter()
            .map(statistical_error_bound)
            .collect(),
        task_loss_bounds: Vec::new(),
    };

    let audit_summary = AuditSummary {
        audit_id: Uuid::new_v4(),
        input_digest,
        external_view_digest,
        policy_version: policy.policy_version.clone(),
        blocked: false,
    };

    Ok(GatewayOutput {
        external_view,
        privacy_report,
        utility_report,
        audit_summary,
        local_mappings: state.local_mappings,
    })
}

fn transform_payload(
    content_type: ContentType,
    payload: Value,
    policy: &Policy,
    key: &[u8],
    state: &mut TransformState,
) -> Value {
    match content_type {
        ContentType::Text => payload
            .as_str()
            .map(|text| Value::String(transform_text(text, policy, key, state)))
            .unwrap_or(payload),
        ContentType::Json | ContentType::CsvRows => transform_value(payload, policy, key, state),
    }
}

fn transform_value(value: Value, policy: &Policy, key: &[u8], state: &mut TransformState) -> Value {
    match value {
        Value::Object(map) => transform_object(map, policy, key, state),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| transform_value(item, policy, key, state))
                .collect(),
        ),
        Value::String(text) => state
            .token_by_original
            .get(&text)
            .map(|token| Value::String(token.clone()))
            .unwrap_or(Value::String(text)),
        other => other,
    }
}

fn transform_object(
    map: Map<String, Value>,
    policy: &Policy,
    key: &[u8],
    state: &mut TransformState,
) -> Value {
    let mut out = Map::new();
    let mut deferred_relations = None;
    for (field_name, value) in map {
        if field_name == "relations" {
            deferred_relations = Some(value);
            continue;
        }

        if let Some(field_policy) = policy.fields.get(&field_name) {
            if field_policy.required_for_task {
                state.required_fields += 1;
            }

            match field_policy.mechanism {
                Mechanism::HmacToken => {
                    let raw = scalar_to_string(&value);
                    state.original_direct_identifiers.push(raw.clone());
                    let token = hmac_token(key, &field_policy.field_type, &raw);
                    state.token_by_original.insert(raw.clone(), token.clone());
                    state.local_mappings.push(LocalMappingEntry {
                        field_name: field_name.clone(),
                        field_type: field_policy.field_type.clone(),
                        token: token.clone(),
                        original_value: raw,
                    });
                    state.preserved_fields += usize::from(field_policy.required_for_task);
                    state.mechanism_evidence.push(MechanismEvidence {
                        field_name: field_name.clone(),
                        field_type: field_policy.field_type.clone(),
                        mechanism: "hmac_token".to_string(),
                        key_domain: Some(policy.key_domain.clone()),
                        token_count: 1,
                    });
                    out.insert(field_name, Value::String(token));
                }
                Mechanism::Suppress => {
                    state.mechanism_evidence.push(MechanismEvidence {
                        field_name: field_name.clone(),
                        field_type: field_policy.field_type.clone(),
                        mechanism: "suppress".to_string(),
                        key_domain: None,
                        token_count: 0,
                    });
                }
                Mechanism::Passthrough => {
                    state.preserved_fields += usize::from(field_policy.required_for_task);
                    out.insert(field_name, transform_value(value, policy, key, state));
                }
                Mechanism::RelativeTime => {
                    state.preserved_fields += usize::from(field_policy.required_for_task);
                    out.insert(
                        field_name.clone(),
                        Value::String(relative_time(&field_name, &value, state)),
                    );
                }
                Mechanism::AddressGeneralize => {
                    state.preserved_fields += usize::from(field_policy.required_for_task);
                    out.insert(
                        field_name,
                        Value::String(generalize_address(
                            &scalar_to_string(&value),
                            field_policy.address_level.as_ref(),
                        )),
                    );
                }
                Mechanism::NumberBucket => {
                    state.preserved_fields += usize::from(field_policy.required_for_task);
                    out.insert(
                        field_name,
                        Value::String(number_bucket(
                            &value,
                            field_policy.bucket_size.unwrap_or(1.0),
                        )),
                    );
                }
            }
        } else {
            out.insert(field_name, transform_value(value, policy, key, state));
        }
    }
    if let Some(value) = deferred_relations {
        out.insert(
            "relations".to_string(),
            transform_value(value, policy, key, state),
        );
    }
    Value::Object(out)
}

fn relative_time(field_name: &str, value: &Value, state: &mut TransformState) -> String {
    let raw = scalar_to_string(value);
    let Ok(timestamp) = DateTime::parse_from_rfc3339(&raw).map(|time| time.with_timezone(&Utc))
    else {
        return "T+unknown".to_string();
    };
    let base = state
        .relative_time_base
        .entry(field_name.to_string())
        .or_insert(timestamp);
    let seconds = timestamp.signed_duration_since(*base).num_seconds();
    if seconds == 0 {
        "T0".to_string()
    } else if seconds > 0 {
        format!("T+{}s", seconds)
    } else {
        format!("T{}s", seconds)
    }
}

fn generalize_address(address: &str, level: Option<&AddressLevel>) -> String {
    match level.unwrap_or(&AddressLevel::City) {
        AddressLevel::Province => split_after_marker(address, '省')
            .or_else(|| split_after_marker(address, '市'))
            .unwrap_or_else(|| generalize_english_address(address, AddressLevel::Province)),
        AddressLevel::City => split_after_marker(address, '市')
            .unwrap_or_else(|| generalize_english_address(address, AddressLevel::City)),
        AddressLevel::District => split_after_marker(address, '区')
            .or_else(|| split_after_marker(address, '县'))
            .unwrap_or_else(|| generalize_english_address(address, AddressLevel::District)),
    }
}

fn split_after_marker(value: &str, marker: char) -> Option<String> {
    value
        .find(marker)
        .map(|index| value[..index + marker.len_utf8()].to_string())
}

fn generalize_english_address(address: &str, level: AddressLevel) -> String {
    let parts = address
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return address.to_string();
    }

    let country = parts.last().copied().unwrap_or_default();
    let region = strip_postal_code(
        parts
            .get(parts.len().saturating_sub(2))
            .copied()
            .unwrap_or(""),
    );
    let city = parts
        .get(parts.len().saturating_sub(3))
        .copied()
        .unwrap_or(region.as_str());

    match level {
        AddressLevel::Province => {
            if country.is_empty() {
                region
            } else {
                format!("{region}, {country}")
            }
        }
        AddressLevel::City | AddressLevel::District => {
            if region.is_empty() {
                city.to_string()
            } else {
                format!("{city}, {region}")
            }
        }
    }
}

fn strip_postal_code(value: &str) -> String {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    if parts.len() <= 1 {
        return value.to_string();
    }
    if parts
        .last()
        .is_some_and(|last| last.chars().any(|ch| ch.is_ascii_digit()))
    {
        parts[..parts.len() - 1].join(" ")
    } else {
        value.to_string()
    }
}

fn number_bucket(value: &Value, bucket_size: f64) -> String {
    let number = value
        .as_f64()
        .unwrap_or_else(|| scalar_to_string(value).parse().unwrap_or(0.0));
    let bucket = if bucket_size > 0.0 {
        (number / bucket_size).floor() * bucket_size
    } else {
        number
    };
    format!("[{:.2},{:.2})", bucket, bucket + bucket_size.max(0.0))
}

fn transform_text(text: &str, policy: &Policy, key: &[u8], state: &mut TransformState) -> String {
    let mut output = text.to_string();
    for (field_name, field_policy) in &policy.fields {
        if !matches!(field_policy.mechanism, Mechanism::HmacToken) {
            continue;
        }
        if let Some(regex) = detector_for(&field_policy.field_type) {
            let mut replacements = Vec::new();
            for found in regex.find_iter(&output) {
                let raw = found.as_str().to_string();
                let token = hmac_token(key, &field_policy.field_type, &raw);
                replacements.push((raw, token));
            }
            for (raw, token) in replacements {
                state.original_direct_identifiers.push(raw.clone());
                state.token_by_original.insert(raw.clone(), token.clone());
                state.local_mappings.push(LocalMappingEntry {
                    field_name: field_name.clone(),
                    field_type: field_policy.field_type.clone(),
                    token: token.clone(),
                    original_value: raw.clone(),
                });
                state.mechanism_evidence.push(MechanismEvidence {
                    field_name: field_name.clone(),
                    field_type: field_policy.field_type.clone(),
                    mechanism: "hmac_token".to_string(),
                    key_domain: Some(policy.key_domain.clone()),
                    token_count: 1,
                });
                output = output.replace(&raw, &token);
            }
        }
    }
    output
}

fn detector_for(field_type: &str) -> Option<Regex> {
    match field_type {
        "phone" => Regex::new(
            r"(?x)(?:\+?\d{1,3}[-.\s]?)?(?:\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}|1[3-9]\d[-\s]?\d{4}[-\s]?\d{4})\b",
        )
        .ok(),
        "national_id" => Regex::new(
            r"(?i)\b\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dx]\b",
        )
        .ok(),
        "email" => Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").ok(),
        "ssn" => Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").ok(),
        "tax_id" => Regex::new(r"\b\d{2}-\d{7}\b").ok(),
        "national_insurance_number" => Regex::new(r"(?i)\b[A-Z]{2}\d{6}[A-Z]\b").ok(),
        "passport_number" => Regex::new(r"(?i)\b[A-Z][0-9]{7,8}\b").ok(),
        "bank_card" | "credit_card" => Regex::new(r"\b(?:\d[ -]?){13,19}\b").ok(),
        "ip_address" => Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").ok(),
        _ => None,
    }
}

fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        other => other.to_string(),
    }
}

fn declared_leakage(policy: &Policy) -> Vec<String> {
    let mut leakage = Vec::new();
    for field in policy.fields.values() {
        if matches!(field.mechanism, Mechanism::HmacToken) && field.preserve_equality {
            leakage.push(format!("{}:equality", field.field_type));
            leakage.push(format!("{}:frequency", field.field_type));
            leakage.push(format!("{}:type", field.field_type));
        }
    }
    leakage.sort();
    leakage.dedup();
    leakage
}

fn residual_risks(policy: &Policy) -> Vec<String> {
    let mut risks = vec![
        "external_linkage_attack_on_unsuppressed_quasi_identifiers".to_string(),
        "contextual_inference_from_remaining_business_fields".to_string(),
    ];
    if policy
        .fields
        .values()
        .any(|field| matches!(field.mechanism, Mechanism::HmacToken))
    {
        risks.push("frequency_analysis_on_deterministic_tokens".to_string());
    }
    risks
}

fn privacy_budget(policy: &Policy) -> PrivacyBudget {
    if policy.statistics.is_empty() {
        return PrivacyBudget::default();
    }

    PrivacyBudget {
        epsilon: Some(policy.statistics.iter().map(|stat| stat.epsilon).sum()),
        delta: Some(policy.statistics.iter().map(|stat| stat.delta).sum()),
        consumed: true,
    }
}

fn constraint_results(payload: &Value, policy: &Policy) -> Vec<VerificationResult> {
    let mut results = Vec::new();

    if policy.constraints.preserve_foreign_keys {
        if policy.constraints.foreign_keys.is_empty() {
            results.push(VerificationResult {
                check: "foreign_key_validity".to_string(),
                passed: false,
                details: "preserve_foreign_keys=true but no foreign_keys declared".to_string(),
            });
        }
        for constraint in &policy.constraints.foreign_keys {
            results.push(foreign_key_result(payload, constraint));
        }
    }

    if policy.constraints.preserve_time_order {
        if policy.constraints.time_orders.is_empty() {
            results.push(VerificationResult {
                check: "time_order_validity".to_string(),
                passed: false,
                details: "preserve_time_order=true but no time_orders declared".to_string(),
            });
        }
        for constraint in &policy.constraints.time_orders {
            results.push(time_order_result(payload, constraint));
        }
    }

    if policy.constraints.preserve_relations {
        if policy.constraints.relations.is_empty() {
            results.push(VerificationResult {
                check: "relation_preservation".to_string(),
                passed: false,
                details: "preserve_relations=true but no relations declared".to_string(),
            });
        }
        for constraint in &policy.constraints.relations {
            results.push(relation_result(payload, constraint));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{AddressLevel, ConstraintPolicy, FieldPolicy};
    use std::collections::BTreeMap;

    #[test]
    fn process_json_replaces_direct_identifier_and_reports_hashes() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "name".to_string(),
            FieldPolicy {
                field_type: "person".to_string(),
                mechanism: Mechanism::HmacToken,
                preserve_equality: true,
                required_for_task: true,
                bucket_size: None,
                address_level: None,
            },
        );
        let policy = Policy {
            policy_version: "test".to_string(),
            task_profile: "contract_review".to_string(),
            key_domain: "local/test".to_string(),
            fields,
            constraints: ConstraintPolicy {
                preserve_relations: true,
                preserve_time_order: false,
                preserve_foreign_keys: false,
                foreign_keys: Vec::new(),
                time_orders: Vec::new(),
                relations: vec![crate::policy::RelationConstraint {
                    table: "relations".to_string(),
                    source_field: "source".to_string(),
                    predicate_field: "predicate".to_string(),
                    target_field: "target".to_string(),
                }],
            },
            statistics: Vec::new(),
        };
        let input = GatewayInput {
            content_type: ContentType::Json,
            payload: serde_json::json!({
                "name":"Alice",
                "role":"buyer",
                "relations":[{"source":"person-1","predicate":"signs","target":"contract-1"}]
            }),
        };
        let output = process_document(input, &policy, b"secret").unwrap();
        let json = serde_json::to_string(&output.external_view).unwrap();
        assert!(!json.contains("Alice"));
        assert!(json.contains("<PERSON_"));
        assert_eq!(output.local_mappings.len(), 1);
        assert!(output.privacy_report.input_digest.starts_with("sha256:"));
        assert!(output.privacy_report.verification_results[0].passed);
        assert_eq!(output.utility_report.entity_preservation.ratio, 1.0);
        assert!(output
            .utility_report
            .constraint_results
            .iter()
            .all(|result| result.passed));
    }

    #[test]
    fn process_text_replaces_detected_phone() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "phone".to_string(),
            FieldPolicy {
                field_type: "phone".to_string(),
                mechanism: Mechanism::HmacToken,
                preserve_equality: true,
                required_for_task: false,
                bucket_size: None,
                address_level: None,
            },
        );
        let policy = Policy {
            policy_version: "test".to_string(),
            task_profile: "support_reply".to_string(),
            key_domain: "local/test".to_string(),
            fields,
            constraints: ConstraintPolicy::default(),
            statistics: Vec::new(),
        };
        let input = GatewayInput {
            content_type: ContentType::Text,
            payload: Value::String("请联系 13800138000 处理".to_string()),
        };
        let output = process_document(input, &policy, b"secret").unwrap();
        let json = serde_json::to_string(&output.external_view).unwrap();
        assert!(!json.contains("13800138000"));
        assert!(json.contains("<PHONE_"));
        assert_eq!(output.local_mappings.len(), 1);
    }

    #[test]
    fn process_text_replaces_english_identifiers() {
        let mut fields = BTreeMap::new();
        for (name, field_type) in [
            ("phone", "phone"),
            ("email", "email"),
            ("ssn", "ssn"),
            ("credit_card", "credit_card"),
        ] {
            fields.insert(
                name.to_string(),
                FieldPolicy {
                    field_type: field_type.to_string(),
                    mechanism: Mechanism::HmacToken,
                    preserve_equality: false,
                    required_for_task: false,
                    bucket_size: None,
                    address_level: None,
                },
            );
        }
        let policy = Policy {
            policy_version: "test".to_string(),
            task_profile: "support_reply".to_string(),
            key_domain: "local/test".to_string(),
            fields,
            constraints: ConstraintPolicy::default(),
            statistics: Vec::new(),
        };
        let input = GatewayInput {
            content_type: ContentType::Text,
            payload: Value::String(
                "Call +1 (415) 555-0198, email alice@example.test, SSN 123-45-6789, card 4111 1111 1111 1111."
                    .to_string(),
            ),
        };
        let output = process_document(input, &policy, b"secret").unwrap();
        let json = serde_json::to_string(&output.external_view).unwrap();
        assert!(!json.contains("415) 555-0198"));
        assert!(!json.contains("alice@example.test"));
        assert!(!json.contains("123-45-6789"));
        assert!(!json.contains("4111 1111 1111 1111"));
        assert!(json.contains("<PHONE_"));
        assert!(json.contains("<EMAIL_"));
        assert!(json.contains("<SSN_"));
        assert!(json.contains("<CREDIT_CARD_"));
    }

    #[test]
    fn process_json_generalizes_english_address() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "address".to_string(),
            FieldPolicy {
                field_type: "address".to_string(),
                mechanism: Mechanism::AddressGeneralize,
                preserve_equality: false,
                required_for_task: false,
                bucket_size: None,
                address_level: Some(AddressLevel::City),
            },
        );
        let policy = Policy {
            policy_version: "test".to_string(),
            task_profile: "address_review".to_string(),
            key_domain: "local/test".to_string(),
            fields,
            constraints: ConstraintPolicy::default(),
            statistics: Vec::new(),
        };
        let input = GatewayInput {
            content_type: ContentType::Json,
            payload: serde_json::json!({
                "address": "100 Market St, San Francisco, CA 94105, USA"
            }),
        };
        let output = process_document(input, &policy, b"secret").unwrap();
        let json = serde_json::to_string(&output.external_view).unwrap();
        assert!(!json.contains("100 Market St"));
        assert!(!json.contains("94105"));
        assert!(json.contains("San Francisco, CA"));
    }
}
