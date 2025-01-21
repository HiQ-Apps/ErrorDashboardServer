use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeatureRequestDTO {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct UpdateFeatureRequestStatusDTO {
    pub id: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeatureRequestDTO {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
