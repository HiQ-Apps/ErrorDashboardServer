use actix_web::HttpResponse;
use bcrypt::{verify, hash};
use chrono::Utc;
use oauth2::basic::BasicClient;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel, TransactionTrait};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::user_dtos::{GoogleUserInfoDTO, ShortUserDTO, ShortUserProfileDTO, UserLoginServiceDTO};
use crate::config::Config;
use crate::models::user_profile_model::{Model as UserProfileModel, Entity as UserProfileEntity};
use crate::models::user_model::{Entity as UserEntity, Model as UserModel};
use crate::models::refresh_token_model::{Entity as RefreshTokenEntity, Model as RefreshTokenModel};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError, RequestError};
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
        let db = &*self.db;
        let now = Utc::now();

        let found_user: Option<UserModel> = UserEntity::find()
            .filter(<UserEntity as sea_orm::EntityTrait>::Column::Email.eq(user_email))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        
        match found_user {
            Some(user) => {
                if user.o_auth_provider != "Custom" {
                    return Err(ServerError::QueryError(QueryError::OAuthTypeError));
                }

                let hashed_password = user.password.clone().ok_or(ServerError::QueryError(QueryError::PasswordNotFound))?;

                let is_valid = verify(&user_password, &hashed_password).map_err(|err| ServerError::from(ExternalError::Bcrypt(err)))?;

                if is_valid {
                    let access_token = create_access_token(user.clone(), &self.configs)?;
                    let refresh_token_dto = create_refresh_token(user.id, &self.configs)?;

                    let refresh_token_model = RefreshTokenModel {
                        user_id: Some(user.id),
                        token: refresh_token_dto.refresh_token.clone(),
                        issued_at: refresh_token_dto.issued_at,
                        expires_at: refresh_token_dto.expires_at,
                        issuer: issuer.to_string(),
                        audience: audience.to_string(),
                        revoked: false,
                        id: Uuid::new_v4(),
                    }.into_active_model();

                    RefreshTokenEntity::insert(refresh_token_model)
                        .exec(db)
                        .await
                        .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

                    let user_profile = UserProfileEntity::find()
                        .filter(<UserProfileEntity as sea_orm::EntityTrait>::Column::UserId.eq(user.id))
                        .one(db)
                        .await
                        .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
                        .ok_or(ServerError::QueryError(QueryError::UserProfileNotFound))?;

                    
                    let user_profile_dto = ShortUserProfileDTO {
                        first_name: user_profile.first_name,
                        last_name: user_profile.last_name,
                        avatar_color: user_profile.avatar_color,
                        updated_at: now
                    };

                    let user_response = UserLoginServiceDTO { 
                        user: ShortUserDTO {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                        },
                        user_profile: user_profile_dto,
                        access_token,
                        refresh_token: refresh_token_dto,
                    };

                    Ok(user_response)
                } else {
                    Err(ServerError::QueryError(QueryError::PasswordIncorrect))
                }
            },
            None => {
                Err(ServerError::QueryError(QueryError::UserNotFound))
            }
        }
    }

    pub async fn google_login(&self, oauth_client: BasicClient ) -> Result<HttpResponse, ServerError> {
        let (auth_url, _csrf_token) = oauth_client
            .authorize_url(oauth2::CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()))
            .add_scope(oauth2::Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
            .url();

        Ok(HttpResponse::Found()
            .append_header(("Location", auth_url.to_string()))
            .finish())
    }

    pub async fn google_callback(&self, user_info: GoogleUserInfoDTO) -> Result<UserLoginServiceDTO, ServerError> {
        let db = &*self.db;

        // Attempt to find the user by email
        let found_user = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Email.eq(user_info.email.clone()))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        // Check if the user exists
        if let Some(user) = found_user {
            // Generate access and refresh tokens for the existing user
            let access_token = create_access_token(user.clone(), &self.configs)?;
            let refresh_token_dto = create_refresh_token(user.id, &self.configs)?;

            // Fetch the user's profile
            let user_profile = UserProfileEntity::find()
                .filter(<UserProfileEntity as EntityTrait>::Column::UserId.eq(user.id))
                .one(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
                .ok_or(ServerError::QueryError(QueryError::UserProfileNotFound))?;

            // Prepare the response DTO
            let user_response = UserLoginServiceDTO {
                user: ShortUserDTO {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                },
                user_profile: ShortUserProfileDTO {
                    first_name: user_profile.first_name,
                    last_name: user_profile.last_name,
                    avatar_color: user_profile.avatar_color,
                    updated_at: Utc::now(),
                },
                access_token,
                refresh_token: refresh_token_dto,
            };
            Ok(user_response)
        } else {
            Err(ServerError::QueryError(QueryError::UserNotFound))
        }
    }

    pub async fn google_register(
        &self,
        user_info: GoogleUserInfoDTO,
    ) -> Result<HttpResponse, ServerError> {
        let db = &*self.db;
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let configs = &*self.configs;

        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let existing_user = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Email.eq(user_info.email.clone()))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        if existing_user.is_some() {
            return Err(ServerError::QueryError(QueryError::UserExists));
        }

        let new_profile_id = Uuid::new_v4();
        let given_name = user_info.given_name.clone();
        let family_name = user_info.family_name.clone();

        let new_user_profile = UserProfileModel {
            id: new_profile_id,
            user_id,
            first_name: Some(user_info.given_name),
            last_name: Some(user_info.family_name),
            avatar_color: "#098585".to_string(),
            created_at: now,
            updated_at: now,
        };

        let new_user = UserModel {
            id: user_id,
            username: user_info.email.clone(),
            email: user_info.email.clone(),
            password: None,
            user_profile_id: new_profile_id,
            o_auth_provider: "Google".to_string(),
            created_at: now,
            updated_at: now,
        };

        let new_user_clone = new_user.clone();

        if let Err(err) = UserProfileEntity::insert(new_user_profile.into_active_model())
            .exec(&transaction)
            .await {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::ExternalError(ExternalError::DB(err)));
            }

        if let Err(err) = UserEntity::insert(new_user.into_active_model())
            .exec(&transaction)
            .await {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::ExternalError(ExternalError::DB(err)));
            }

        let access_token = create_access_token(new_user_clone.clone(), &configs)?;
        let refresh_token_dto = create_refresh_token(new_user_clone.id, &configs)?;

        Ok(HttpResponse::Ok().json(UserLoginServiceDTO {
            user: ShortUserDTO {
                id: user_id,
                username: user_info.email.clone(),
                email: user_info.email.clone(),
            },
            user_profile: ShortUserProfileDTO {
                first_name: Some(given_name),
                last_name: Some(family_name),
                avatar_color: "#098585".to_string(),
                updated_at: now,
            },
            access_token,
            refresh_token: refresh_token_dto,
        }))
    }


    pub async fn register(
        &self,
        user_name: String,
        user_email: String,
        user_pass: String,
    ) -> Result<UserLoginServiceDTO, ServerError> {
        let db = &*self.db;
        let configs = &*self.configs;
        let issuer = &configs.jwt_issuer;
        let audience = &configs.jwt_audience;

        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let hash_cost = configs.hash_cost.parse().unwrap_or(bcrypt::DEFAULT_COST);
        let uid = Uuid::new_v4();
        let now = Utc::now();

        let hashed_pass = hash(user_pass, hash_cost).map_err(|err| ServerError::ExternalError(ExternalError::Bcrypt(err)))?;

        let initialize_user_profile = UserProfileModel {
            id: Uuid::new_v4(),
            user_id: uid,
            first_name: None,
            last_name: None,
            avatar_color: "#098585".to_string(),
            created_at: now,
            updated_at: now,
        
        };

        let user = UserModel {
            id: uid,
            username: user_name,
            email: user_email,
            password: Some(hashed_pass),
            o_auth_provider: "Custom".to_string(),
            user_profile_id: initialize_user_profile.id,
            created_at: now,
            updated_at: now,
        };

        let active_user_profile = initialize_user_profile.clone().into_active_model();

        if let Err(err) = UserProfileEntity::insert(active_user_profile)
            .exec(&transaction)
            .await {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::ExternalError(ExternalError::DB(err)));
            }

        let active_user_model = user.clone().into_active_model();

        if let Err(err) = UserEntity::insert(active_user_model)
            .exec(&transaction)
            .await {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::ExternalError(ExternalError::DB(err)));
            }

        let access_token = create_access_token(user.clone(), configs)?;
        let refresh_token_dto = create_refresh_token(user.id, configs)?;

        let refresh_token_model = RefreshTokenModel {
            user_id: Some(user.id),
            token: refresh_token_dto.refresh_token.clone(),
            issued_at: refresh_token_dto.issued_at,
            expires_at: refresh_token_dto.expires_at,
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            revoked: false,
            id: Uuid::new_v4(),
        }.into_active_model();

        if let Err(err) = RefreshTokenEntity::insert(refresh_token_model)
            .exec(&transaction)
            .await {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::from(ExternalError::DB(err)));
            }

        transaction.commit().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let user_response = UserLoginServiceDTO {
            user: ShortUserDTO {
                id: uid,
                username: user.username.clone(),
                email: user.email.clone(),
            },
            user_profile: ShortUserProfileDTO {
                first_name: initialize_user_profile.first_name,
                last_name: initialize_user_profile.last_name,
                avatar_color: "#098585".to_string(),
                updated_at: now,
            },
            access_token,
            refresh_token: refresh_token_dto,
        };

        Ok(user_response)
    }

    pub async fn verify_user(&self, user_id: Uuid, user_pass: String) -> Result<(), ServerError> {
        let db = &*self.db;

        let found_user: Option<UserModel> = UserEntity::find()
            .filter(<UserEntity as sea_orm::EntityTrait>::Column::Id
            .eq(user_id))
            .one(db)
            .await
            .map_err(|err|ServerError::from(ExternalError::DB(err)))?;

        match found_user {
            Some(user) => {
                if let Some(stored_password) = &user.password {
                    let is_valid = verify(&user_pass, stored_password)
                        .map_err(|err| ServerError::from(ExternalError::Bcrypt(err)))?;
                    if is_valid {
                        Ok(())
                    } else {
                        Err(ServerError::QueryError(QueryError::PasswordIncorrect))
                    }
                } else {
                    Err(ServerError::QueryError(QueryError::PasswordNotSet))
                }
            },
            None => Err(ServerError::QueryError(QueryError::UserNotFound))
        }
    }

    pub async fn find_by_token(&self, token: &str) -> Result<Option<RefreshTokenModel>, ServerError> {
        RefreshTokenEntity::find()
            .filter(<RefreshTokenEntity as sea_orm::EntityTrait>::Column::Token.eq(token))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))
    }
    
    pub async fn process_token_refresh(&self, token: &str) -> Result<UserLoginServiceDTO, ServerError> {
        let db = &*self.db;
        let configs = &*self.configs;
        let issuer = &configs.jwt_issuer;
        let audience = &configs.jwt_audience;
        let now = Utc::now();

        let mut token_model = self
            .find_by_token(token)
            .await?
            .ok_or(ServerError::RequestError(RequestError::InvalidToken))?;

        token_model.revoked = true;
        let active_token_model = token_model.clone().into_active_model();

        RefreshTokenEntity::update(active_token_model)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let user_id_uuid = token_model.user_id.ok_or(ServerError::RequestError(RequestError::MissingUserID))?;

        let new_refresh_token_dto = create_refresh_token(user_id_uuid, configs)?;
        let new_refresh_token_model = RefreshTokenModel {
            user_id: Some(user_id_uuid),
            token: new_refresh_token_dto.refresh_token.clone(),
            issued_at: new_refresh_token_dto.issued_at,
            expires_at: new_refresh_token_dto.expires_at,
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            revoked: false,
            id: Uuid::new_v4(),
        };

        let active_refresh_token_model = new_refresh_token_model.clone().into_active_model();

        RefreshTokenEntity::insert(active_refresh_token_model)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let refreshed_access_token = refresh_access_token_util(new_refresh_token_model, db, configs).await?;

        let found_user = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Id.eq(user_id_uuid))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

        let found_user_profile = UserProfileEntity::find()
            .filter(<UserProfileEntity as EntityTrait>::Column::UserId.eq(found_user.id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::UserProfileNotFound))?;

        let user_response = UserLoginServiceDTO {
            user: ShortUserDTO {
                id: found_user.id,
                username: found_user.username,
                email: found_user.email,
            },
            user_profile: ShortUserProfileDTO {
                first_name: found_user_profile.first_name,
                last_name: found_user_profile.last_name,
                avatar_color: found_user_profile.avatar_color,
                updated_at: now,
            },
            access_token: refreshed_access_token,
            refresh_token: new_refresh_token_dto,
        };

        Ok(user_response)
        
    }
}
