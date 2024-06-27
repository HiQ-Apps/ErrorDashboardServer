use chrono::Utc;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel, ActiveValue};
use shared_types::user_dtos::{ShortUserDTO, ShortUserProfileDTO, UpdateUserProfileDTO};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::models::user_model::Entity as UserEntity;
use crate::models::user_profile_model::Entity as UserProfileEntity;
use crate::shared::utils::errors::{ServerError, QueryError, ExternalError};


pub struct UserService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>, configs:Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
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
        let get_base_user_query = UserProfileEntity::find_by_id(uid)
            .one(&*self.db).await;

        match get_base_user_query {
            Ok(Some(user)) => {
                let user_profile_dto = ShortUserProfileDTO {
                    first_name: user.first_name,
                    last_name: user.last_name,
                    avatar_color: user.avatar_color,
                    updated_at: user.updated_at,
                };

                Ok(user_profile_dto)
            },
            Ok(None) => Err(ServerError::from(QueryError::UserNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err)))
        }
    }

    pub async fn update_user_profile(&self, uid: Uuid, update_user_profile: UpdateUserProfileDTO) -> Result<ShortUserProfileDTO, ServerError> {
        let db = &*self.db;
        let now = Utc::now();

        let update_query = UserProfileEntity::find()
            .filter(<UserProfileEntity as EntityTrait>::Column::UserId.eq(uid))
            .one(db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)));
        
        let mut user_profile = match update_query {
            Ok(Some(user_profile)) => user_profile.into_active_model(),
            Ok(None) => return Err(ServerError::from(QueryError::UserNotFound)),
            Err(err) => return Err(ServerError::from(err))
        };

        if let Some(avatar_color) = update_user_profile.avatar_color {
            user_profile.avatar_color = ActiveValue::Set(avatar_color);
        };

        if let Some(first_name) = update_user_profile.first_name {
            user_profile.first_name = ActiveValue::Set(Some(first_name));
        };

        if let Some(last_name) = update_user_profile.last_name {
            user_profile.last_name = ActiveValue::Set(Some(last_name));
        };

        user_profile.updated_at = ActiveValue::Set(now);

        let updated_user_profile = match user_profile.update(db).await {
            Ok(user) => user,
            Err(err) => return Err(ServerError::from(ExternalError::DB(err)))
        };


        let user_profile_dto = ShortUserProfileDTO {
            first_name: updated_user_profile.first_name,
            last_name: updated_user_profile.last_name,
            avatar_color: updated_user_profile.avatar_color,
            updated_at: updated_user_profile.updated_at,
        };

        Ok(user_profile_dto)
    }

    pub async fn delete_user(&self, uid: Uuid) -> Result<Uuid, ServerError> {
        let delete_query = UserEntity::delete_by_id(uid)
            .exec(&*self.db)
            .await;
            
        
        match delete_query {
            Ok(_) => Ok(uid),
            Err(err) => Err(ServerError::from(ExternalError::DB(err)))
        }
    }
}
