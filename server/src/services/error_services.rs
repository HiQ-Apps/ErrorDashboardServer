use chrono::{Utc, DateTime, Duration};
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel, DatabaseConnection};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use shared_types::error_dtos::{AggregateErrorDto, CreateErrorDto, ErrorDto, ShortErrorDto, UpdateErrorDto};
use crate::models::error_model::{Entity as ErrorEntity, Model as ErrorModel};
use crate::models::user_namespace_junction_model::{Entity as UserNamespaceJunctionEntity, Model as UserNamespaceJunctionModel};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError};

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
    ) -> Result<ShortErrorDto, ServerError> {
        let now = Utc::now();
        let create_error = ErrorModel {
            id: Uuid::new_v4(),
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
        };
        
        ErrorEntity::insert(create_error.clone().into_active_model())
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        
        Ok(ShortErrorDto {
            id: create_error.id,
            status_code: create_error.status_code,
            message: create_error.message,
            resolved: create_error.resolved,
            namespace_id: create_error.namespace_id,
        })
    }

    pub async fn get_error_by_id(
        &self,
        id: Uuid,
    ) -> Result<ErrorDto, ServerError> {
        let found_error = ErrorEntity::find()
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::Id.eq(id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::ErrorNotFound))?;

        Ok(ErrorDto {
            id: found_error.id,
            status_code: found_error.status_code,
            user_affected: found_error.user_affected,
            path: found_error.path,
            line: found_error.line,
            message: found_error.message,
            stack_trace: found_error.stack_trace,
            namespace_id: found_error.namespace_id,
            resolved: found_error.resolved,
            created_at: found_error.created_at,
            updated_at: found_error.updated_at,
        })
    }

    pub async fn update_error(
        &self,
        error: UpdateErrorDto,
    ) -> Result<UpdateErrorDto, ServerError> {
        let now = Utc::now();

        let found_error = ErrorEntity::find()
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::Id.eq(error.id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::ErrorNotFound))?;

        let mut update_error = found_error.clone();

        if let Some(resolved) = error.resolved {
            update_error.resolved = resolved;
        }

        update_error.updated_at = now;

        ErrorEntity::update(update_error.clone().into_active_model())
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(UpdateErrorDto {
            id: update_error.id,
            resolved: Some(update_error.resolved),
        })
    }

    // Get all errors for a specific namespace by namespace_id
    // Using the created_at date column, aggregate a param given time frame ( every 4 hours ) from a param given start time ( 2 years )
    // return the count of errors and the time frame until the current time
    pub async fn get_aggregate_errors_by_date(
        &self,
        namespace_id: Uuid,
        start_time: DateTime<Utc>,
        time_interval: i64,
    ) -> Result<Vec<AggregateErrorDto>, ServerError> {
        let now = Utc::now();
        let mut current_time = start_time;
        let mut aggregated_errors: Vec<AggregateErrorDto> = Vec::new();

        while current_time < now {
            let end_time = current_time + Duration::hours(time_interval as i64);
            let count = ErrorEntity::find()
                .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId.eq(namespace_id))
                .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt.gt(current_time))
                .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt.lt(end_time))
                .count(&*self.db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

            aggregated_errors.push(AggregateErrorDto {
                count,
                time: current_time,
            });

            current_time = end_time;
        }

        Ok(aggregated_errors)
    }
}
