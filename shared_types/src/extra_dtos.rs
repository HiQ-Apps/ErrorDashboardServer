use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct PaginationParams {
    pub offset: u64,
    pub limit: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct TimeParams {
    pub start_time: DateTime<Utc>,
    pub time_interval_hours: i64,
}

