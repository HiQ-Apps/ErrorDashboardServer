use serde::{Serialize, Deserialize};
use serde_valid::Validate;
use uuid::Uuid;

use super::auth_dtos::RefreshTokenDTO;


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
pub struct UserLoginDTO {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserLoginServiceDTO {
    pub user: ShortUserDTO,
    pub access_token: String,
    pub refresh_token: RefreshTokenDTO
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct UserResponseDTO {
    pub user: ShortUserDTO,
    pub access_token: String,
}

impl From<UserLoginServiceDTO> for UserResponseDTO {
    fn from(service_dto: UserLoginServiceDTO) -> Self {
        UserResponseDTO {
            user: service_dto.user,
            access_token: service_dto.access_token,
        }
    }
}
