use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct TagDTO {
    pub id: Uuid,
    pub tag_key: String,
    pub tag_value: String,
    pub error_id: Uuid,
    pub tag_color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ShortTagDTO {
    pub id: Uuid,
    pub tag_key: String,
    pub tag_value: String,
    pub tag_color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateTagDTO {
    pub tag_key: String,
    pub tag_value: String,
    pub tag_color: String,
    pub error_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateTagClientNoIdDTO {
    pub tag_key: String,
    pub tag_value: String,
    pub error_id: Uuid,
}

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ShortTagNoIdDTO {
    pub tag_key: String,
    pub tag_value: String,
    pub tag_color: String,
}
