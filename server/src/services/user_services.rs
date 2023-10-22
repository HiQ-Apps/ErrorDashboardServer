use sea_orm::{entity::prelude::*, EntityTrait};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::models::user_model::{Entity as UserEntity, Model as UserModel};
use crate::shared::utils::errors::ServerError;

pub struct UserService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>, configs:Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }


    pub async fn get_user(&self, uid: Uuid) -> Option<UserModel> {
        UserEntity::find_by_id(uid)
            .one(&*self.db).await.ok()
            .flatten()
    }


    pub async fn delete_user(&self, uid: Uuid) -> Result<(), ServerError> {
        UserEntity::delete_by_id(uid)
            .exec(&*self.db).await?;
        Ok(())
    }
}
