use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateNamespaceDto {
    pub service_name: String,
    pub environment_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateNamespaceDto {
    pub id: Uuid,
    pub active: Option<bool>,
    pub service_name: Option<String>,
    pub environment_type: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_secret: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NamespaceDto {
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: Uuid,
    pub client_secret: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}
