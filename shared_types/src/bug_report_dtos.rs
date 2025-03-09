use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateBugReportDTO {
    pub issue: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct UpdateBugReportStatusDTO {
    pub id: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BugReportDTO {
    pub id: i32,
    pub issue: String,
    pub description: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
