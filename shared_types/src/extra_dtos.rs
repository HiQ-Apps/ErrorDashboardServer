use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub offset: u64,
    pub limit: u64,
}
