use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue, EntityTrait, IntoActiveModel, DatabaseConnection, TransactionError, TransactionTrait};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use shared_types::error_dtos::CreateErrorDto;
use crate::models::error_model::{Entity as ErrorEntity, Model as ErrorModel};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError, RequestError};

pub struct ErrorService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>
}

impl ErrorService {
        pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_error(
        &self,
        error: CreateErrorDto,
    ) -> Result<Uuid, ServerError> {
        let uid = Uuid::new_v4();
        let now = Utc::now();

        let create_error = ErrorModel {
            id: uid,
            status_code: error.status_code,
            user_affected: error.user_affected,
            path: error.path,
            line: error.line,
            message: error.message,
            stack_trace: error.stack_trace,
            resolved: false,
            namespace_id: error.namespace_id,
            created_at: now,
            updated_at: now,
        }.into_active_model();

        println!("{:?}", create_error);
        ErrorEntity::insert(create_error).exec(&*self.db).await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(uid)
    }

}