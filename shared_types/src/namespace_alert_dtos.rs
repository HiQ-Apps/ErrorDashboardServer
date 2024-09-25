use uuid::Uuid;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateNamespaceAlertRequestDTO {
    pub namespace_id: Uuid,
    pub alert_method: String,
    pub error_name: Option<String>,
    pub path: Option<String>,
    pub line: Option<String>,
    pub message: Option<String>,
    pub stack_trace: Option<String>,
    pub count_threshold: Option<i32>,
    pub time_window: Option<i64>,
    pub unresolved_time_threshold: Option<i64>,
    pub rate_threshold: Option<i32>,
    pub rate_time_window: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceAlertDTO {
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub alert_method: String,
    pub error_name: Option<String>,
    pub path: Option<String>,
    pub line: Option<String>,
    pub message: Option<String>,
    pub stack_trace: Option<String>,
    pub count_threshold: Option<i32>,
    pub time_window: Option<i64>,
    pub unresolved_time_threshold: Option<i64>,
    pub rate_threshold: Option<i32>,
    pub rate_time_window: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ShortNamespaceAlertDTO {
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub alert_method: String,
    pub error_name: Option<String>,
    pub path: Option<String>,
    pub line: Option<String>,
    pub message: Option<String>,
    pub stack_trace: Option<String>,
    pub count_threshold: Option<i32>,
    pub time_window: Option<i64>,
    pub unresolved_time_threshold: Option<i64>,
    pub rate_threshold: Option<i32>,
    pub rate_time_window: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct DeleteNamespaceAlertRequestDTO {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct GetNamespaceAlertByNamespaceIdRequestDTO {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct GetNamespaceAlertByUserIdRequestDTO {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNamespaceAlertRequestDTO {
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub alert_method: String,
    pub error_name: Option<String>,
    pub path: Option<String>,
    pub line: Option<String>,
    pub message: Option<String>,
    pub stack_trace: Option<String>,
    pub count_threshold: Option<i32>,
    pub time_window: Option<i64>,
    pub unresolved_time_threshold: Option<i64>,
    pub rate_threshold: Option<i32>,
    pub rate_time_window: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceAlertSubscriptionRequestDTO {
    pub user_id: Uuid,
    pub namespace_id: Uuid,
    pub namespace_alert_id: Uuid,
}

