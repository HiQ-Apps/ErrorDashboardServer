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
    pub id: Uuid,
    pub tag_key: String,
    pub tag_value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateTagDto {
    pub tag_key: String,
    pub tag_value: String,
    pub error_id: Uuid,
}

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ShortTagDtoNoId {
    pub tag_key: String,
    pub tag_value: String,
}
