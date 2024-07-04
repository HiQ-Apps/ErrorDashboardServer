use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct CreateNamespaceDTO {
    pub service_name: String,
    pub environment_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct GetNamespaceResponseDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: Uuid,
    pub client_secret: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>

}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UpdateNamespaceDTO {
    pub id: Uuid,
    pub active: Option<bool>,
    pub service_name: Option<String>,
    pub environment_type: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_secret: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct NamespaceDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: Uuid,
    pub client_secret: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}


#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct ShortNamespaceDTO {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
}
