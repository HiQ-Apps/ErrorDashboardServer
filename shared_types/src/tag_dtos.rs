use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct TagDto {
    pub id: Uuid,
    pub tag_key: String,
    pub tag_value: String,
    pub error_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ShortTagDto {
    pub tag_key: String,
    pub tag_value: String,
}
