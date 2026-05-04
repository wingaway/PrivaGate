use crate::policy::{
    ForeignKeyConstraint, RelationConstraint, StatisticPolicy, TimeOrderConstraint,
};
use crate::report::{PreservationMetric, VerificationResult};
use serde_json::Value;
use std::collections::BTreeSet;

pub fn no_direct_identifier_residue(
    original_values: &[String],
    external_json: &str,
) -> VerificationResult {
    let leaked: Vec<&String> = original_values
        .iter()
        .filter(|value| !value.is_empty() && external_json.contains(value.as_str()))
        .collect();

    VerificationResult {
        check: "direct_identifier_residue".to_string(),
        passed: leaked.is_empty(),
        details: format!("residual_direct_identifier_count={}", leaked.len()),
    }
}

pub fn preservation(required: usize, preserved: usize) -> PreservationMetric {
    let ratio = if required == 0 {
        1.0
    } else {
        preserved as f64 / required as f64
    };

    PreservationMetric {
        required,
        preserved,
        ratio,
    }
}

pub fn foreign_key_result(
    payload: &Value,
    constraint: &ForeignKeyConstraint,
) -> VerificationResult {
    let source_values =
        table_field_values(payload, &constraint.source_table, &constraint.source_field);
    let target_values =
        table_field_values(payload, &constraint.target_table, &constraint.target_field)
            .into_iter()
            .collect::<BTreeSet<_>>();
    let missing = source_values
        .iter()
        .filter(|value| !target_values.contains(*value))
        .count();

    VerificationResult {
        check: "foreign_key_validity".to_string(),
        passed: missing == 0,
        details: format!(
            "{}.{} -> {}.{}, source_values={}, missing={}",
            constraint.source_table,
            constraint.source_field,
            constraint.target_table,
            constraint.target_field,
            source_values.len(),
            missing
        ),
    }
}

pub fn time_order_result(payload: &Value, constraint: &TimeOrderConstraint) -> VerificationResult {
    let values = table_field_values(payload, &constraint.table, &constraint.field);
    let order_values = values
        .iter()
        .map(|value| relative_time_seconds(value).unwrap_or(0))
        .collect::<Vec<_>>();
    let violations = order_values
        .windows(2)
        .filter(|pair| pair[0] > pair[1])
        .count();

    VerificationResult {
        check: "time_order_validity".to_string(),
        passed: violations == 0,
        details: format!(
            "{}.{}, values={}, violations={}",
            constraint.table,
            constraint.field,
            values.len(),
            violations
        ),
    }
}

fn relative_time_seconds(value: &str) -> Option<i64> {
    if value == "T0" {
        return Some(0);
    }
    value
        .strip_prefix("T+")
        .and_then(|tail| tail.strip_suffix('s'))
        .and_then(|seconds| seconds.parse::<i64>().ok())
        .or_else(|| {
            value
                .strip_prefix('T')
                .and_then(|tail| tail.strip_suffix('s'))
                .and_then(|seconds| seconds.parse::<i64>().ok())
        })
}

pub fn relation_result(payload: &Value, constraint: &RelationConstraint) -> VerificationResult {
    let rows = table_rows(payload, &constraint.table);
    let complete = rows
        .iter()
        .filter(|row| {
            row.get(&constraint.source_field).is_some()
                && row.get(&constraint.predicate_field).is_some()
                && row.get(&constraint.target_field).is_some()
        })
        .count();
    let missing = rows.len().saturating_sub(complete);

    VerificationResult {
        check: "relation_preservation".to_string(),
        passed: missing == 0,
        details: format!(
            "{}({}, {}, {}), rows={}, incomplete={}",
            constraint.table,
            constraint.source_field,
            constraint.predicate_field,
            constraint.target_field,
            rows.len(),
            missing
        ),
    }
}

pub fn statistical_error_bound(policy: &StatisticPolicy) -> String {
    let absolute_error_bound = (1.0 / policy.beta).ln() * policy.sensitivity / policy.epsilon;
    format!(
        "{}: mechanism={:?} epsilon={} delta={} beta={} sensitivity={} absolute_error_bound={:.6}",
        policy.name,
        policy.mechanism,
        policy.epsilon,
        policy.delta,
        policy.beta,
        policy.sensitivity,
        absolute_error_bound
    )
}

fn table_field_values(payload: &Value, table: &str, field: &str) -> Vec<String> {
    table_rows(payload, table)
        .into_iter()
        .filter_map(|row| row.get(field))
        .map(value_to_key)
        .collect()
}

fn table_rows<'a>(payload: &'a Value, table: &str) -> Vec<&'a serde_json::Map<String, Value>> {
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
