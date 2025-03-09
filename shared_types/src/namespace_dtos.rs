use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateNamespaceDTO {
    pub service_name: String,
    pub environment_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetNamespaceResponseDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: Uuid,
    pub client_secret: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNamespaceDTO {
    pub id: Uuid,
    pub active: Option<bool>,
    pub service_name: Option<String>,
    pub environment_type: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_secret: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: Uuid,
    pub client_secret: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ShortNamespaceDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetNamespacesByUserResponseDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct InviteUserRequestDTO {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRoleRequestDTO {
    pub user_id: Uuid,
    pub role: String,
}
