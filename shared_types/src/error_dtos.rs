use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use uuid::Uuid;

use super::tag_dtos::{TagDto, ShortTagDto};

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ShortErrorDto {
    pub id: Uuid,
    pub status_code: i16,
    pub message: String,
    pub resolved: bool,
    pub tags: Option<Vec<ShortTagDto>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateErrorDto {
    pub id: Uuid,
    pub status_code: i16,
    pub message: String,
    pub resolved: bool,
    pub namespace_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateErrorResponse {
    pub id: Uuid,
    pub status_code: i16,
    pub message: String,
    pub resolved: bool,
    pub namespace_id: Uuid,
}


#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ErrorDto {
    pub id: Uuid,
    pub status_code: i16,
    pub user_affected: String,
    pub path: String,
    pub line: i32,
    pub message: String,
    pub stack_trace: String,
    pub namespace_id: Uuid,
    pub resolved: bool,
    pub tags: Option<Vec<TagDto>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateErrorRequest {
    pub status_code: i16,
    pub user_affected: String,
    pub path: String,
    pub line: i32,
    pub stack_trace: String,
    pub message: String,
    pub namespace_id: Uuid,
    pub tags: Option<Vec<ShortTagDto>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateErrorDto {
    pub id: Uuid,
    pub resolved: Option<bool>,
    pub tags: Option<Vec<TagDto>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AggregateErrorDto {
    pub count: i64,
    pub time: DateTime<Utc>,
}
