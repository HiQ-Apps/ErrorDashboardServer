use actix_web::http::StatusCode;
use bcrypt::{verify, hash};
use chrono::Utc;
use sea_orm::Database;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::namespace_dtos::NamespaceDto;
use crate::config::Config;
use crate::models::namespace_model::{NamespaceEntity, Model as NamespaceModel};
use crate::models::user_model::{UserEntity, Model as UserModel};
use crate::models::user_namespace_junction_model::{UserNamespaceJunctionEntity, Model as UserNamespaceJunctionModel};
use crate::shared::utils::errors::{ServerError, HttpError};

pub struct NamespaceService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>
}

impl NamespaceService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_namespace(
        &self,
        namespace_service_name: String,
        environment_type: String
    ) -> Result<Uuid, ServerError> {
        let uid = Uuid::new_v4();
        let new_client_secret = Uuid::new_v4().to_string();
        let new_client_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let namespace = NamespaceModel {
            id: uid,
            active: false,
            service_name: namespace_service_name,
            environment_type: environment_type,
            client_id: new_client_secret,
            client_secret: new_client_id,
            error_records: None,
            created_at: now,
            updated_at: None
        }.into_active_model();

        NamespaceEntity::insert(namespace).exec(&*self.db).await?;
        
        Ok(uid)
    }

    pub async fn get_namespace_by_id(&self, uid: Uuid) -> Result<(), ServerError> {
        match NamespaceEntity::find().filter(NamespaceEntity::Id.eq(uid)).one(&*self.db).await {
            Ok(namespace) => {
                Ok(())
            },
            Err(_) => {
                Err(ServerError::NamespaceNotFound)
            }
        }
    }
    
    pub async fn get_namespaces_by_user_id(&self, user_id: str) -> Result<(), ServerError> {
        match NamespaceEntity::find().filter(NamespaceEntity::userId.eq(user_id).all()).await {
            Ok(namespaces) => {
                Ok(())
            },
            Err(_) => {
                Err(ServerError::UserNotFound)
            }
        }
    }

    
    pub async fn update_namespace(&self) -> Result<(), ServerError> {
    }

    
    pub async fn delete_namespace(&self) -> Result<(), ServerError> {
    }
}
