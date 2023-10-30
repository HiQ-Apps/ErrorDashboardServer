use actix_web::http::StatusCode;
use bcrypt::{verify, hash};
use chrono::Utc;
use sea_orm::Database;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::namespace_dtos::NamespaceDto;
use crate::config::Config;
use crate::models::namespace_model::{Entity as NamespaceEntity, Model as NamespaceModel};
use crate::models::user_model::{Entity as UserEntity, Model as UserModel};
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
    ) -> Result<(), ServerError> {
        let uid = Uuid::new_v4();
        let client_secret = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let now = Utc::now().naive_local();

        let namespace = NamespaceModel {
            id: uid,
            active: false,
            service_name: namespace_service_name,
            environment_type: environment_type,
            client_id: client_id,
            client_secret: client_secret,
            error_records: None,
            created_at: now,
            updated_at: None
        }.into_active_model();



    }

    pub async fn get_namespace_by_id(&self, uid: Uuid) -> Result<(), ServerError> {
    }
    
    pub async fn get_namespaces_by_user_id(&self) -> Result<(), ServerError> {
    }

    
    pub async fn update_namespace(&self) -> Result<(), ServerError> {
    }

    
    pub async fn delete_namespace(&self) -> Result<(), ServerError> {
    }
}
