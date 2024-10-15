use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use uuid::Uuid;

use super::tag_dtos::{TagDTO, ShortTagNoIdDTO, ShortTagDTO, CreateTagClientNoIdDTO, CreateTagRequestDTO};

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ShortErrorDTO {
    pub id: Uuid,
    pub status_code: i16,
    pub message: String,
    pub resolved: bool,
    pub tags: Option<Vec<ShortTagNoIdDTO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetAggregatedStatusErrorDTO {
    pub status_code: i16,
    pub aggregated_tags: Vec<ShortTagNoIdDTO>,
    pub user_affected_count: i32,
    pub error_count: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetAggregatedMessageErrorDTO {
    pub message: String,
    pub aggregated_tags: Vec<ShortTagNoIdDTO>,
    pub user_affected_count: i32,
    pub error_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetAggregatedLineErrorDTO {
    pub line: i32,
    pub aggregated_tags: Vec<ShortTagNoIdDTO>,
    pub user_affected_count: i32,
    pub error_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AggregatedResult {
    ByTags(Vec<TagAggregatedErrorDTO>),
    ByLine(Vec<GetAggregatedLineErrorDTO>),
    ByMessage(Vec<GetAggregatedMessageErrorDTO>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagAggregatedErrorDTO {
    pub tag: ShortTagNoIdDTO,
    pub user_affected_count: i32,
    pub error_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateErrorDTO {
    pub id: Uuid,
    pub message: String,
    pub resolved: bool,
    pub namespace_id: Uuid,
    pub stack_trace: String,
    pub user_affected: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateErrorResponse {
    pub id: Uuid,
    pub message: String,
    pub namespace_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDTO {
    pub id: Uuid,
    pub user_affected: String,
    pub message: String,
    pub path: String,
    pub line: i32,
    pub stack_trace: String,
    pub namespace_id: Uuid,
    pub resolved: bool,
    pub tags: Option<Vec<ShortTagDTO>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AggregateIndividualErrorDTO {
    pub id: Uuid,
    pub user_affected: String,
    pub message: String,
    pub path: String,
    pub line: i32,
    pub stack_trace: String,
    pub namespace_id: Uuid,
    pub resolved: bool,
    pub tags: Option<Vec<ShortTagNoIdDTO>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMetaDTO {
    pub id: Uuid,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GroupedAggregateErrorCountDTO {
    pub group_key: String,
    pub count: i64
}


#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateErrorRequest {
    pub user_affected: String,
    pub stack_trace: String,
    pub message: String,
    pub tags: Option<Vec<CreateTagClientNoIdDTO>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateErrorDTO {
    pub id: Uuid,
    pub resolved: Option<bool>,
    pub tags: Option<Vec<TagDTO>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AggregateErrorDTO {
    pub count: i64,
    pub time: DateTime<Utc>,
}

