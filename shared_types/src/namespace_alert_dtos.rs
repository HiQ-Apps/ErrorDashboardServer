use uuid::Uuid;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateNamespaceAlertRequestDTO {
    pub namespace_id: Uuid,
    pub alert_method: String,
    pub discord_channel_id: Option<String>,
    pub path: Option<String>,
    pub line: Option<i32>,
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
pub struct NamespaceAlertDTO<'a> {
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub alert_method: &'a str,
    pub discord_channel_id: Option<&'a str>,
    pub triggered: bool,
    pub path: Option<&'a str>,
    pub line: Option<i32>,
    pub message: Option<&'a str>,
    pub stack_trace: Option<&'a str>,
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
    pub discord_channel_id: Option<String>,
    pub triggered: bool,
    pub path: Option<String>,
    pub line: Option<i32>,
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
pub struct UpdateNamespaceAlertRequestDTO {
    pub namespace_id: Uuid,
    pub alert_method: Option<String>,
    pub discord_channel_id: Option<String>,
    pub triggered: bool,
    pub path: Option<String>,
    pub line: Option<i32>,
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
