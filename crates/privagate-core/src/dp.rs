use crate::digest::sha256_json;
use crate::policy::{StatisticMechanism, StatisticPolicy};
use crate::report::{PrivacyBudget, VerificationResult};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticOutput {
    pub input_digest: String,
    pub privacy_budget: PrivacyBudget,
    pub results: Vec<StatisticResult>,
    pub verification_results: Vec<VerificationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticResult {
    pub name: String,
    pub mechanism: String,
    pub table: String,
    pub raw_value: Value,
    pub noised_value: Value,
    pub epsilon: f64,
    pub delta: f64,
    pub beta: f64,
    pub sensitivity: f64,
    pub absolute_error_bound: f64,
}

pub fn compute_statistics(
    payload: &Value,
    policies: &[StatisticPolicy],
    key: &[u8],
) -> Result<StatisticOutput, serde_json::Error> {
    let input_digest = sha256_json(payload)?;
    let mut results = Vec::new();
    let mut verification_results = Vec::new();

    for policy in policies {
        match policy.mechanism {
            StatisticMechanism::LaplaceCount => {
                let raw_value = table_count(payload, &policy.table) as f64;
                let scale = policy.sensitivity / policy.epsilon;
                let noise = deterministic_laplace_noise(key, &input_digest, &policy.name, scale);
                let absolute_error_bound =
                    (1.0 / policy.beta).ln() * policy.sensitivity / policy.epsilon;
                results.push(StatisticResult {
                    name: policy.name.clone(),
                    mechanism: "laplace_count".to_string(),
                    table: policy.table.clone(),
                    raw_value: number_value(raw_value),
                    noised_value: number_value(raw_value + noise),
                    epsilon: policy.epsilon,
                    delta: policy.delta,
                    beta: policy.beta,
                    sensitivity: policy.sensitivity,
                    absolute_error_bound,
                });
                verification_results.push(VerificationResult {
                    check: "dp_budget_recorded".to_string(),
                    passed: policy.epsilon > 0.0 && policy.delta >= 0.0,
                    details: format!(
                        "{} epsilon={} delta={} sensitivity={}",
                        policy.name, policy.epsilon, policy.delta, policy.sensitivity
                    ),
                });
            }
            StatisticMechanism::LaplaceHistogram => {
                let raw_bins = histogram(payload, &policy.table, policy.group_field.as_deref());
                let scale = policy.sensitivity / policy.epsilon;
                let mut noised_bins = Map::new();
                let mut raw_json = Map::new();
                for (bin, count) in raw_bins {
                    raw_json.insert(bin.clone(), number_value(count as f64));
                    let noise = deterministic_laplace_noise(
                        key,
                        &input_digest,
                        &format!("{}:{bin}", policy.name),
                        scale,
                    );
                    noised_bins.insert(bin, number_value(count as f64 + noise));
                }
                let absolute_error_bound =
                    (1.0 / policy.beta).ln() * policy.sensitivity / policy.epsilon;
                results.push(StatisticResult {
                    name: policy.name.clone(),
                    mechanism: "laplace_histogram".to_string(),
                    table: policy.table.clone(),
                    raw_value: Value::Object(raw_json),
                    noised_value: Value::Object(noised_bins),
                    epsilon: policy.epsilon,
                    delta: policy.delta,
                    beta: policy.beta,
                    sensitivity: policy.sensitivity,
                    absolute_error_bound,
                });
                verification_results.push(VerificationResult {
                    check: "dp_budget_recorded".to_string(),
                    passed: policy.epsilon > 0.0
                        && policy.delta >= 0.0
                        && policy.group_field.is_some(),
                    details: format!(
                        "{} epsilon={} delta={} sensitivity={} group_field={:?}",
                        policy.name,
                        policy.epsilon,
                        policy.delta,
                        policy.sensitivity,
                        policy.group_field
                    ),
                });
            }
            StatisticMechanism::LaplaceMean => {
                let raw_value = mean(
                    payload,
                    &policy.table,
                    policy.value_field.as_deref(),
                    policy.lower,
                    policy.upper,
                );
                let scale = policy.sensitivity / policy.epsilon;
                let noise = deterministic_laplace_noise(key, &input_digest, &policy.name, scale);
                let absolute_error_bound =
                    (1.0 / policy.beta).ln() * policy.sensitivity / policy.epsilon;
                results.push(StatisticResult {
                    name: policy.name.clone(),
                    mechanism: "laplace_mean".to_string(),
                    table: policy.table.clone(),
                    raw_value: number_value(raw_value),
                    noised_value: number_value(raw_value + noise),
                    epsilon: policy.epsilon,
                    delta: policy.delta,
                    beta: policy.beta,
                    sensitivity: policy.sensitivity,
                    absolute_error_bound,
                });
                verification_results.push(VerificationResult {
                    check: "dp_budget_recorded".to_string(),
                    passed: policy.epsilon > 0.0
                        && policy.delta >= 0.0
                        && policy.value_field.is_some()
                        && policy.lower.is_some()
                        && policy.upper.is_some(),
                    details: format!(
                        "{} epsilon={} delta={} sensitivity={} value_field={:?} lower={:?} upper={:?}",
                        policy.name, policy.epsilon, policy.delta, policy.sensitivity, policy.value_field, policy.lower, policy.upper
                    ),
                });
            }
        }
    }

    Ok(StatisticOutput {
        input_digest,
        privacy_budget: PrivacyBudget {
            epsilon: Some(policies.iter().map(|policy| policy.epsilon).sum()),
            delta: Some(policies.iter().map(|policy| policy.delta).sum()),
            consumed: !policies.is_empty(),
        },
        results,
        verification_results,
    })
}

fn table_count(payload: &Value, table: &str) -> usize {
    payload
        .get(table)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn histogram(payload: &Value, table: &str, group_field: Option<&str>) -> BTreeMap<String, usize> {
    let Some(group_field) = group_field else {
        return BTreeMap::new();
    };
    let mut bins = BTreeMap::new();
    for row in table_rows(payload, table) {
        if let Some(value) = row.get(group_field) {
            *bins.entry(value_to_key(value)).or_insert(0) += 1;
        }
    }
    bins
}

fn mean(
    payload: &Value,
    table: &str,
    value_field: Option<&str>,
    lower: Option<f64>,
    upper: Option<f64>,
) -> f64 {
    let (Some(value_field), Some(lower), Some(upper)) = (value_field, lower, upper) else {
        return 0.0;
    };
    let values = table_rows(payload, table)
        .into_iter()
        .filter_map(|row| row.get(value_field))
        .filter_map(Value::as_f64)
        .map(|value| value.clamp(lower, upper))
        .collect::<Vec<_>>();
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn table_rows<'a>(payload: &'a Value, table: &str) -> Vec<&'a Map<String, Value>> {
    payload
        .get(table)
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(Value::as_object).collect::<Vec<_>>())
        .unwrap_or_default()
}

fn value_to_key(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        other => other.to_string(),
    }
}

fn number_value(value: f64) -> Value {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

fn deterministic_laplace_noise(key: &[u8], input_digest: &str, name: &str, scale: f64) -> f64 {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of any size");
    mac.update(input_digest.as_bytes());
    mac.update(b":");
    mac.update(name.as_bytes());
    let bytes = mac.finalize().into_bytes();
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    let integer = u64::from_be_bytes(raw);
    let unit = ((integer as f64) + 0.5) / ((u64::MAX as f64) + 1.0);
    let centered = unit - 0.5;
    if centered < 0.0 {
        scale * (1.0 + 2.0 * centered).ln()
    } else {
        -scale * (1.0 - 2.0 * centered).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::StatisticPolicy;

    #[test]
    fn laplace_count_is_recomputable() {
        let policy = StatisticPolicy {
            name: "orders".to_string(),
            table: "orders".to_string(),
            group_field: None,
            value_field: None,
            lower: None,
            upper: None,
            mechanism: StatisticMechanism::LaplaceCount,
            epsilon: 0.5,
            delta: 0.0,
            beta: 0.01,
            sensitivity: 1.0,
        };
        let payload = serde_json::json!({"orders":[{}, {}, {}]});
        let a = compute_statistics(&payload, std::slice::from_ref(&policy), b"secret").unwrap();
        let b = compute_statistics(&payload, &[policy], b"secret").unwrap();
        assert_eq!(a.results[0].raw_value, serde_json::json!(3.0));
        assert_eq!(a.results[0].noised_value, b.results[0].noised_value);
        assert!(a.privacy_budget.consumed);
    }
}
