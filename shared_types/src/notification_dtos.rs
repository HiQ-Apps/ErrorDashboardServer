use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateNotificationDTO {
    pub title: String,
    pub source: String,
    pub text: String,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDTO {
    pub id: Uuid,
    pub title: String,
    pub source: String,
    pub user_id: Uuid,
    pub text: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

