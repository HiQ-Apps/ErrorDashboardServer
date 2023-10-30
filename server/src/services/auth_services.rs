use actix_web::http::StatusCode;
use bcrypt::{verify, hash};
use chrono::Utc;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::user_dtos::{ShortUserDTO, UserLoginServiceDTO};

use crate::config::Config;
use crate::models::user_model::{Entity as UserEntity, Model as UserModel};
use crate::models::refresh_token_model::{Entity as RefreshTokenEntity, Model as RefreshTokenModel};
use crate::shared::utils::errors::{ServerError, HttpError};
use crate::shared::utils::jwt::{create_access_token, create_refresh_token, refresh_access_token_util};

pub struct AuthService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl AuthService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }


    pub async fn login(
        &self,
        user_email: String,
        user_password: String
    ) -> Result<UserLoginServiceDTO, ServerError> {
        let issuer = &self.configs.jwt_issuer;
        let audience = &self.configs.jwt_audience;

        let found_user: Option<UserModel> = UserEntity::find()
            .filter(<UserEntity as sea_orm::EntityTrait>::Column::Email
            .eq(user_email))
            .one(&*self.db)
            .await.map_err(ServerError::DBError)?;

        match found_user {
            Some(user) => {
                let is_valid = verify(&user_password, &user.password).map_err(ServerError::from)?;
                if is_valid {
                    let access_token = create_access_token(user.clone(), &self.configs)?;
                    let refresh_token_dto = create_refresh_token(user.id.to_string(),&self.configs)?;

                    let refresh_token_model = RefreshTokenModel {
                        user_id: user.id,
                        token: refresh_token_dto.refresh_token.clone(),
                        issued_at: refresh_token_dto.issued_at,
                        expires_at: refresh_token_dto.expires_at,
                        issuer: issuer.to_string(),
                        audience: audience.to_string(),
                        revoked: false,
                        id: Uuid::new_v4(),
                    }.into_active_model();

                    RefreshTokenEntity::insert(refresh_token_model)
                        .exec(&*self.db)
                        .await?;

                    let user_response = UserLoginServiceDTO { 
                        user: ShortUserDTO {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        },
                        access_token,
                        refresh_token: refresh_token_dto
                    };

                    Ok(user_response)
                
                } else {
                    Err(ServerError::WebError(HttpError { status: StatusCode::BAD_REQUEST, message: "Invalid password".to_string() }))
                }
            },
            None => Err(ServerError::WebError(HttpError { status: StatusCode::NOT_FOUND, message: "User not found".to_string() }))
        }
    }


    pub async fn register(
        &self, 
        user_name: String,
        user_email: String,
        user_pass: String
    ) -> Result<Uuid, ServerError> {
        let hash_cost = self.configs.hash_cost.parse().unwrap();
        let uid = Uuid::new_v4();
        let now = Utc::now().naive_local();
        let hashed_pass = hash(user_pass, hash_cost).unwrap();

        let user = UserModel {
            id: uid,
            username: user_name,
            email: user_email,
            password: hashed_pass,
            created_at: now,
            updated_at: None
        }.into_active_model();

        UserEntity::insert(user).exec(&*self.db).await?;
        
        Ok(uid)
    }


    pub async fn refresh_access_token(&self, refresh_token: String) -> Result<String, ServerError> {
        let refresh_token_model: RefreshTokenModel = serde_json::from_str(&refresh_token)
            .map_err(ServerError::JsonError)?;

        match refresh_access_token_util(refresh_token_model, &self.db, &self.configs).await {
            Ok(token) => Ok(token),
            Err(err) => Err(err),
        }
    }


    pub async fn find_by_token(&self, token: &str) -> Result<Option<RefreshTokenModel>, ServerError> {
        RefreshTokenEntity::find()
            .filter(<RefreshTokenEntity as sea_orm::EntityTrait>::Column::Token.eq(token))
            .one(&*self.db)
            .await
            .map_err(ServerError::DBError)
    }


    pub async fn process_token_refresh(&self, token: &str) -> Result<String, ServerError> {
        let issuer = &self.configs.jwt_issuer;
        let audience = &self.configs.jwt_audience;

        let mut token_model = self
            .find_by_token(token)
            .await?
            .ok_or(ServerError::InvalidToken)?;

        token_model.revoked = true;
        
        let active_token_model = token_model.clone().into_active_model();
        RefreshTokenEntity::update(active_token_model).exec(&*self.db).await?;
        
        let new_refresh_token_dto = create_refresh_token(token_model.user_id.to_string(), &self.configs)?;
        let new_refresh_token_model = RefreshTokenModel {
                        user_id: token_model.user_id,
                        token: new_refresh_token_dto.refresh_token.clone(),
                        issued_at: new_refresh_token_dto.issued_at,
                        expires_at: new_refresh_token_dto.expires_at,
                        issuer: issuer.to_string(),
                        audience: audience.to_string(),
                        revoked: false,
                        id: Uuid::new_v4(),
                    }.into_active_model();

        RefreshTokenEntity::insert(new_refresh_token_model)
            .exec(&*self.db)
            .await?;
        
        Ok(new_refresh_token_dto.refresh_token)
    }
}
