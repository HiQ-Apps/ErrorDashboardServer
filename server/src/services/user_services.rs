use sea_orm::{entity::prelude::*, EntityTrait};
use shared_types::user_dtos::ShortUserDTO;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::models::user_model::Entity as UserEntity;
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

    // Delete junctions as well
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
