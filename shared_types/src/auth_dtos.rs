use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshTokenDTO {
    pub refresh_token: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub jwt_iss: String,
    pub jwt_aud: String,
    pub revoked: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,
    pub iss: String,
    pub aud: String,
    pub data: Option<JsonValue>
}
