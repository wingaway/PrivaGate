use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_version: String,
    pub task_profile: String,
    pub key_domain: String,
    pub fields: BTreeMap<String, FieldPolicy>,
    #[serde(default)]
    pub constraints: ConstraintPolicy,
    #[serde(default)]
    pub statistics: Vec<StatisticPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPolicy {
    pub field_type: String,
    pub mechanism: Mechanism,
    #[serde(default)]
    pub preserve_equality: bool,
    #[serde(default)]
    pub required_for_task: bool,
    #[serde(default)]
    pub bucket_size: Option<f64>,
    #[serde(default)]
    pub address_level: Option<AddressLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    HmacToken,
    Suppress,
    Passthrough,
    RelativeTime,
    AddressGeneralize,
    NumberBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressLevel {
    Province,
    City,
    District,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintPolicy {
    #[serde(default)]
    pub preserve_relations: bool,
    #[serde(default)]
    pub preserve_time_order: bool,
    #[serde(default)]
    pub preserve_foreign_keys: bool,
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKeyConstraint>,
    #[serde(default)]
    pub time_orders: Vec<TimeOrderConstraint>,
    #[serde(default)]
    pub relations: Vec<RelationConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyConstraint {
    pub source_table: String,
    pub source_field: String,
    pub target_table: String,
    pub target_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOrderConstraint {
    pub table: String,
    pub field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConstraint {
    pub table: String,
    pub source_field: String,
    pub predicate_field: String,
    pub target_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticPolicy {
    pub name: String,
    pub table: String,
    #[serde(default)]
    pub group_field: Option<String>,
    #[serde(default)]
    pub value_field: Option<String>,
    #[serde(default)]
    pub lower: Option<f64>,
    #[serde(default)]
    pub upper: Option<f64>,
    pub mechanism: StatisticMechanism,
    pub epsilon: f64,
    #[serde(default)]
    pub delta: f64,
    #[serde(default = "default_beta")]
    pub beta: f64,
    #[serde(default = "default_count_sensitivity")]
    pub sensitivity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatisticMechanism {
    LaplaceCount,
    LaplaceHistogram,
    LaplaceMean,
}

fn default_beta() -> f64 {
    0.01
}

fn default_count_sensitivity() -> f64 {
    1.0
}
