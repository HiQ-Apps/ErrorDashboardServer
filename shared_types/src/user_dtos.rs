use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

use super::auth_dtos::RefreshTokenDTO;

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileDTO {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar_color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserCreateDTO {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ShortUserDTO {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MemberListDTO {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ShortUserProfileDTO {
    pub avatar_color: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub updated_at: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequestDTO {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserProfileDTO {
    pub avatar_color: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<String>,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginDTO {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordPath {
    pub id: Uuid,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PasswordDTO {
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginServiceDTO {
    pub user: ShortUserDTO,
    pub user_profile: ShortUserProfileDTO,
    pub access_token: String,
    pub refresh_token: RefreshTokenDTO
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserResponseDTO {
    pub user: ShortUserDTO,
    pub user_profile: ShortUserProfileDTO,
    pub access_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GoogleUserInfoDTO {
    // Google user ID
    pub sub: String,
    // User's email address
    pub email: String,
    // Whether the email is verified
    pub email_verified: bool,
    // Full name
    pub name: String,      
    // First name   
    pub given_name: String,
    // Last name
    pub family_name: String,
    // URL to the user's profile picture
    pub picture: String,
}


impl From<UserLoginServiceDTO> for UserResponseDTO {
    fn from(service_dto: UserLoginServiceDTO) -> Self {
        UserResponseDTO {
            user: service_dto.user,
            user_profile: service_dto.user_profile,
            access_token: service_dto.access_token,
        }
    }
}
