use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct QueryParams {
    pub offset: u64,
    pub limit: u64,
    pub group_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct PaginationParams {
    pub offset: u64,
    pub limit: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct TimeParams {
    pub start_time: NaiveDate,
    pub time_interval_minutes: i64,
    pub timezone: String,
}
