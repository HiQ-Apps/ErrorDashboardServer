use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_valid::Validate;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenDTO {
    pub refresh_token: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub jwt_iss: String,
    pub jwt_aud: String,
    pub revoked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub sub: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,
    pub iss: String,
    pub aud: String,
    pub data: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenServiceDTO {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VerifyUserDTO {
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}
