use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel, DatabaseConnection};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use shared_types::error_dtos::{AggregateErrorDto, CreateErrorDto, ErrorDto, ShortErrorDto, UpdateErrorDto};
use crate::models::error_model::{Entity as ErrorEntity, Model as ErrorModel};
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

    pub async fn get_aggregate_errors_by_date(
        &self,
        namespace_id: Uuid,
        selected_date: NaiveDate,
        time_interval: i64,
        timezone: String,
    ) -> Result<Vec<AggregateErrorDto>, ServerError> {
        let user_tz: Tz = timezone.parse().map_err(|_| ServerError::QueryError(QueryError::InvalidTimestamp))?;

        let start_time_naive = selected_date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| ServerError::QueryError(QueryError::InvalidTimestamp))?;
        let start_time_local = user_tz.from_local_datetime(&start_time_naive)
            .single()
            .ok_or_else(|| ServerError::QueryError(QueryError::InvalidTimestamp))?;
        let end_time_local = start_time_local + Duration::days(1);
        println!("start time local: {:?}", start_time_local); 
        println!("end time local: {:?}", end_time_local);

        // Convert local times to UTC
        let start_time_utc = start_time_local.with_timezone(&Utc);
        let end_time_utc = end_time_local.with_timezone(&Utc);

        println!("Start time: {}", start_time_utc);
        println!("End time: {}", end_time_utc);

        let errors = ErrorEntity::find()
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt.gt(start_time_utc))
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt.lt(end_time_utc))
            .all(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let mut error_map: HashMap<DateTime<Utc>, i64> = HashMap::new();

        println!("{:?}", errors);

        for error in errors {
            let time_bucket_seconds = (error.created_at.timestamp() / (time_interval * 60)) * (time_interval * 60);
            let time_bucket = match Utc.timestamp_opt(time_bucket_seconds, 0) {
                chrono::LocalResult::Single(dt) => dt,
                _ => return Err(ServerError::QueryError(QueryError::InvalidTimestamp)),
            };

            *error_map.entry(time_bucket).or_insert(0) += 1;
        }

        let mut aggregated_errors: Vec<AggregateErrorDto> = error_map
            .into_iter()
            .map(|(time, count)| AggregateErrorDto { count, time })
            .collect();

        aggregated_errors.sort_by(|a, b| a.time.cmp(&b.time));
        println!("{:?}", aggregated_errors);
        Ok(aggregated_errors)
    }
}
