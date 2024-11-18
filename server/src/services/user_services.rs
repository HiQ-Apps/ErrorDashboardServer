use bcrypt::hash;
use chrono::Utc;
use sea_orm::{entity::prelude::*, EntityTrait, ActiveValue, TransactionTrait, IntoActiveModel, ConnectionTrait};
use shared_types::user_dtos::{BaseUserDTO, GetUsersAdminResponseDTO, ResetPasswordRequestDTO, ShortUserDTO, ShortUserProfileDTO, UpdateUserProfileDTO, UserAdminDTO, UserProfileDTO};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::models::user_model::{Entity as UserEntity, ActiveModel as UserActiveModel};
use crate::models::user_profile_model::{Entity as UserProfileEntity, ActiveModel as UserProfileActiveModel};
use crate::shared::utils::errors::{ServerError, QueryError, ExternalError};
use crate::shared::utils::mailing::{send_email, EmailContent};


pub struct UserService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>, configs:Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn get_all_users(&self) -> Result<GetUsersAdminResponseDTO, ServerError> {
        let db = &*self.db;
        let users = UserEntity::find().all(db).await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;
        let user_profiles = UserProfileEntity::find().all(db).await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let mut user_list = Vec::new();

        for user in users {
            let user_profile = user_profiles.iter().find(|profile| profile.user_id == user.id);

            if let Some(profile) = user_profile {
                let user_profile = UserAdminDTO {
                    user: BaseUserDTO {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        verified: user.verified,
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    },
                    user_profile: UserProfileDTO {
                        id: user.id,
                        first_name: profile.first_name.clone(),
                        last_name: profile.last_name.clone(),
                        avatar_color: profile.avatar_color.clone(),
                        role: profile.role.clone(),
                        created_at: profile.created_at,
                        updated_at: profile.updated_at,
                    }
                };
        
                user_list.push(user_profile);
            }
        }

        Ok(GetUsersAdminResponseDTO{users: user_list})
    }

    pub async fn get_user(&self, uid: Uuid) -> Result<ShortUserDTO, ServerError> {
        let get_base_user_query = UserEntity::find_by_id(uid)
            .one(&*self.db).await;

        match get_base_user_query {
            Ok(Some(user)) => {
                let user_dto = ShortUserDTO {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                };

                Ok(user_dto)
            },
            Ok(None) => Err(ServerError::from(QueryError::UserNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err)))
        }
    }

    pub async fn get_user_profile(&self, uid: Uuid) -> Result<ShortUserProfileDTO, ServerError> {
        let get_base_user_query = UserProfileEntity::find()
            .filter(<UserProfileEntity as EntityTrait>::Column::UserId.eq(uid))
            .one(&*self.db)
            .await;

        match get_base_user_query {
            Ok(Some(user)) => {
                let user_profile_dto = ShortUserProfileDTO {
                    first_name: user.first_name,
                    last_name: user.last_name,
                    role: user.role,
                    avatar_color: user.avatar_color,
                    updated_at: user.updated_at,
                };

                Ok(user_profile_dto)
            },
            Ok(None) => Err(ServerError::from(QueryError::UserNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err)))
        }
    }

    pub async fn check_user_verification(&self, uid: Uuid) -> Result<bool, ServerError> {
        let db = &*self.db;

        let user_query = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Id.eq(uid))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let user = match user_query {
            Some(user_query) => user_query,
            None => return Err(ServerError::from(QueryError::UserNotFound))
        };

        if !user.verified {
            return Err(ServerError::from(QueryError::UserNotVerified))
        } else {
            return Ok(true)
        }
    }

    pub async fn forgot_password(&self, req: ResetPasswordRequestDTO) -> Result<(), ServerError> {
        let db = &*self.db;
        let configs = &*self.configs;

        let user_query = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Email.eq(req.email))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let user = match user_query {
            Some(user_query) => user_query,
            None => return Err(ServerError::from(QueryError::UserNotFound))
        };

        let dynamic_forget_pass_url = format!("{}/forget-password/{}", configs.domain, user.id);
        
        let content = EmailContent {
            greeting: "Password Change".to_string(),
            main_message: "Forgot your password?".to_string(),
            body: format!("It seems you're trying to update your password, if this is not you, please disregard this message. Otherwise, please click this link to update your password: <a href=\"{}\" style=\"color: #1a73e8;\"> Forgot my password </a>", dynamic_forget_pass_url),
            dynamic_content: None,
        };

        send_email(configs, &user.email, "Higuard Password Change", &content).map_err(|err| ServerError::from(err))?;

        Ok(())
    }

    pub async fn update_password(&self, uid: Uuid, email: String, password: String) -> Result<(), ServerError> {
        let db = &*self.db;
        let configs = &*self.configs;
        let now = Utc::now();
        let hash_cost = configs.hash_cost.parse().unwrap();

        let hashed_pass = hash(password, hash_cost).map_err(|err| ServerError::ExternalError(ExternalError::Bcrypt(err)))?;

        let user_query = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Id.eq(uid))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let user = match user_query {
            Some(user) => user,
            None => return Err(ServerError::from(QueryError::UserNotFound))
        };

        if user.email != email {
            return Err(ServerError::from(QueryError::UserNotFound))
        }

        let mut active_user = user.into_active_model();

        active_user.password = ActiveValue::Set(Some(hashed_pass));
        active_user.updated_at = ActiveValue::Set(now);

        if let Err(err) = active_user.update(db).await {
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }

        Ok(())
    }

    pub async fn verify_user(&self, uid: Uuid) -> Result<(), ServerError> {
        let db = &*self.db;
        let now = Utc::now();
        let configs = &*self.configs;
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let update_query = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Id.eq(uid))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let user = match update_query {
            Some(user) => user,
            None => return Err(ServerError::from(QueryError::UserNotFound)),
        };

        let active_user = UserActiveModel {
            id: ActiveValue::Set(user.id),
            email: ActiveValue::Set(user.email),
            user_profile_id: ActiveValue::Set(user.user_profile_id),
            username: ActiveValue::Set(user.username),
            password: ActiveValue::Set(user.password),
            verified: ActiveValue::Set(true),
            o_auth_provider: ActiveValue::Set(user.o_auth_provider),
            created_at: ActiveValue::Set(user.created_at),
            updated_at: ActiveValue::Set(now),
        };

        match active_user.update(&transaction).await {
            Ok(user) => user,
            Err(err) => {
                transaction.rollback().await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;
                return Err(ServerError::from(ExternalError::DB(err)))
            }
        };

        transaction.commit().await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        Ok(())
    }

    pub async fn update_user_profile(&self, uid: Uuid, update_user_profile: UpdateUserProfileDTO) -> Result<ShortUserProfileDTO, ServerError> {
        let db = &*self.db;
        let now = Utc::now();
        let configs = &*self.configs;
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        let hash_cost = configs.hash_cost.parse().unwrap_or(bcrypt::DEFAULT_COST);
        let mut profile_updated = false;
        let mut user_updated = false;

        let update_query = UserProfileEntity::find()
            .filter(<UserProfileEntity as EntityTrait>::Column::UserId.eq(uid))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;
        
        let user_profile = match update_query {
            Some(profile) => profile,
            None => return Err(ServerError::from(QueryError::UserNotFound)),
        };


        let mut active_user_profile = UserProfileActiveModel {
            id: ActiveValue::Set(user_profile.id),
            user_id: ActiveValue::Set(user_profile.user_id),
            first_name: user_profile.first_name.map_or(ActiveValue::Unchanged(None), |v| ActiveValue::Unchanged(Some(v))),
            last_name: user_profile.last_name.map_or(ActiveValue::Unchanged(None), |v| ActiveValue::Unchanged(Some(v))),
            avatar_color: ActiveValue::Set(user_profile.avatar_color),
            role: ActiveValue::Set(user_profile.role),
            created_at: ActiveValue::Set(user_profile.created_at),
            updated_at: ActiveValue::Unchanged(user_profile.updated_at),
        };

        let update_user_query = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Id.eq(uid))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let user = match update_user_query {
            Some(user) => user,
            None => return Err(ServerError::from(QueryError::UserNotFound)),
        };

        let mut active_user = UserActiveModel {
            id: ActiveValue::Set(user.id),
            email: ActiveValue::Unchanged(user.email),
            user_profile_id: ActiveValue::Set(user.user_profile_id),
            username: ActiveValue::Unchanged(user.username),
            password: ActiveValue::Unchanged(user.password),
            verified: ActiveValue::Unchanged(user.verified),
            o_auth_provider: ActiveValue::Unchanged(user.o_auth_provider),
            created_at: ActiveValue::Set(user.created_at),
            updated_at: ActiveValue::Unchanged(user.updated_at),
        };

        if let Some(ref first_name) = update_user_profile.first_name {
            if !first_name.is_empty() {
                active_user_profile.first_name = ActiveValue::Set(Some(first_name.clone()));
                profile_updated = true;
            }
        }

        if let Some(ref last_name) = update_user_profile.last_name {
            if !last_name.is_empty() {
                active_user_profile.last_name = ActiveValue::Set(Some(last_name.clone()));
                profile_updated = true;
            }
        }

        if let Some(ref avatar_color) = update_user_profile.avatar_color {
            if !avatar_color.is_empty() {
                active_user_profile.avatar_color = ActiveValue::Set(avatar_color.clone());
                profile_updated = true;
            }
        }

        if let Some(ref username) = update_user_profile.username {
            if !username.is_empty() {
                active_user.username = ActiveValue::Set(username.clone());
                user_updated = true;
            }
        }

        if let Some(ref password) = update_user_profile.password {
            if !password.is_empty() {
                let hashed_password = hash(password, hash_cost).map_err(|err| ServerError::ExternalError(ExternalError::Bcrypt(err)))?;
                active_user.password = ActiveValue::Set(Some(hashed_password));
                user_updated = true;
            }
        }

        if profile_updated {
            active_user_profile.updated_at = ActiveValue::Set(now);
        }

        if user_updated {
            active_user.updated_at = ActiveValue::Set(now);
        }

        let _updated_user = match active_user.update(&transaction).await {
            Ok(user) => user,
            Err(err) => {
                transaction.rollback().await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;
                return Err(ServerError::from(ExternalError::DB(err)))
            }
        };

        let updated_user_profile = match active_user_profile.update(&transaction).await {
            Ok(user) => user,
            Err(err) => {
                transaction.rollback().await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;
                return Err(ServerError::from(ExternalError::DB(err)))}
        };

        transaction.commit().await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let user_profile_dto = ShortUserProfileDTO {
            first_name: updated_user_profile.first_name,
            last_name: updated_user_profile.last_name,
            avatar_color: updated_user_profile.avatar_color,
            role: updated_user_profile.role,
            updated_at: updated_user_profile.updated_at,
        };

        Ok(user_profile_dto)
    }

    pub async fn get_user_role_by_user_id(&self, user_id: Uuid) -> Result<String, ServerError> {
        let user_profile = UserProfileEntity::find()
            .filter(<UserProfileEntity as EntityTrait>::Column::UserId.eq(user_id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        match user_profile {
            Some(user_profile) => Ok(user_profile.role),
            None => Err(ServerError::from(QueryError::UserNotFound))
        }
    }
}
