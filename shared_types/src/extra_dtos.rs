use serde::{Serialize, Deserialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct PaginationParams {
    pub offset: u64,
    pub limit: u64,
}
