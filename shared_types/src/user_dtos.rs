use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

use super::auth_dtos::RefreshTokenDTO;

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserProfileDTO {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar_color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserCreateDTO {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct ShortUserDTO {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct ShortUserProfileDTO {
    pub avatar_color: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub updated_at: DateTime<Utc>
}


#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UpdateUserProfileDTO {
    pub avatar_color: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<String>,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserLoginDTO {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserLoginServiceDTO {
    pub user: ShortUserDTO,
    pub user_profile: ShortUserProfileDTO,
    pub access_token: String,
    pub refresh_token: RefreshTokenDTO
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserResponseDTO {
    pub user: ShortUserDTO,
    pub user_profile: ShortUserProfileDTO,
    pub access_token: String,
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
