use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_version: String,
    pub task_profile: String,
    pub key_domain: String,
    pub fields: BTreeMap<String, FieldPolicy>,
    #[serde(default)]
    pub task_contracts: BTreeMap<String, TaskContract>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskContract {
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default)]
    pub allowed_adapter_classes: Vec<AdapterClass>,
    #[serde(default)]
    pub promotion_utility: PromotionUtilityProfile,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionUtilityProfile {
    #[serde(default)]
    pub require_required_fields: bool,
    #[serde(default)]
    pub required_constraint_checks: Vec<UtilityConstraintCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    HmacToken,
    LocalOnly,
    Suppress,
    Passthrough,
    RelativeTime,
    AddressGeneralize,
    NumberBucket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterClass {
    Reserved,
    LocalPrivate,
    ExternalZeroRetention,
    ExternalStandard,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityConstraintCheck {
    ForeignKeyValidity,
    TimeOrderValidity,
    RelationPreservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskContractIssue {
    pub field_name: String,
    pub details: String,
}

impl Policy {
    pub fn task_contract_issues(&self, task_profile: &str) -> Vec<TaskContractIssue> {
        let Some(contract) = self.task_contracts.get(task_profile) else {
            return Vec::new();
        };

        let mut issues = Vec::new();
        for field_name in &contract.required_fields {
            match self.fields.get(field_name) {
                Some(field_policy) if matches!(field_policy.mechanism, Mechanism::LocalOnly) => {
                    issues.push(TaskContractIssue {
                        field_name: field_name.clone(),
                        details: format!(
                            "task_profile={task_profile} requires field={field_name} but policy keeps it local_only"
                        ),
                    });
                }
                Some(_) => {}
                None => issues.push(TaskContractIssue {
                    field_name: field_name.clone(),
                    details: format!(
                        "task_profile={task_profile} requires field={field_name} but no field policy is declared"
                    ),
                }),
            }
        }
        issues
    }

    pub fn adapter_contract_issues(
        &self,
        task_profile: &str,
        adapter_class: &AdapterClass,
    ) -> Vec<String> {
        let Some(contract) = self.task_contracts.get(task_profile) else {
            return Vec::new();
        };
        if contract.allowed_adapter_classes.is_empty() {
            return Vec::new();
        }
        if contract
            .allowed_adapter_classes
            .iter()
            .any(|allowed| allowed == adapter_class)
        {
            return Vec::new();
        }

        vec![format!(
            "task_profile={task_profile} requires adapter_class in [{}] but current adapter_class={}",
            contract
                .allowed_adapter_classes
                .iter()
                .map(AdapterClass::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            adapter_class.as_str(),
        )]
    }
}

impl AdapterClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdapterClass::Reserved => "reserved",
            AdapterClass::LocalPrivate => "local_private",
            AdapterClass::ExternalZeroRetention => "external_zero_retention",
            AdapterClass::ExternalStandard => "external_standard",
        }
    }
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
