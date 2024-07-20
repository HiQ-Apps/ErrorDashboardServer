use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sea_orm::sea_query::Query;
use sea_orm::{entity::prelude::*, EntityTrait, IntoActiveModel, QueryOrder, QuerySelect, Condition, DatabaseConnection};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use shared_types::tag_dtos::{TagDTO, ShortTagDTO, CreateTagRequestDTO};
use shared_types::error_dtos::{AggregateErrorDTO, CreateErrorDTO, CreateErrorRequest, ErrorDTO, ErrorMetaDTO, UpdateErrorDTO};
use crate::models::error_model::{Entity as ErrorEntity, Model as ErrorModel};
use crate::models::error_tag_model::{Entity as TagEntity, Model as TagModel, ActiveModel as ActiveTagModel};
use crate::models::namespace_model::Entity as NamespaceEntity;
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError};
use crate::shared::utils::parse::{parse_stack_trace, StackTraceInfo};

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
        error: CreateErrorRequest,
        namespace_client_id: Uuid,
    ) -> Result<CreateErrorDTO, ServerError> {
        let now = Utc::now();

        let mut stack_trace_info: StackTraceInfo = StackTraceInfo::default();
        let error_stack_trace = error.stack_trace.clone();

        // Add  the rest later
        match parse_stack_trace(&error_stack_trace) {
            Ok(info) => stack_trace_info = info,
            Err(err) => println!("Failed to parse stack trace: {}", err),
        }

        let found_namespace = NamespaceEntity::find()
            .filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::ClientId.eq(namespace_client_id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::NamespaceNotFound))?;


        let create_error = ErrorModel {
            id: Uuid::new_v4(),
            user_affected: error.user_affected,
            path: stack_trace_info.file_path,
            line: stack_trace_info.line_number,
            message: error.message,
            stack_trace: error_stack_trace,
            resolved: false,
            namespace_id: found_namespace.id,
            created_at: now,
            updated_at: now,
        };
        
        ErrorEntity::insert(create_error.clone().into_active_model())
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;


        let mut return_tags: Vec<CreateTagRequestDTO> = Vec::new();

        if let Some(tags) = error.tags {
            for tag in tags {
                let tag_dto = CreateTagRequestDTO {
                    tag_key: tag.tag_key,
                    tag_value: tag.tag_value,
                    error_id: create_error.id,
                };
                return_tags.push(tag_dto.clone());

                let tag_model: ActiveTagModel = tag_dto.into();

                TagEntity::insert(tag_model)
                    .exec(&*self.db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
            }
        }
        
        Ok(CreateErrorDTO {
            id: create_error.id,
            message: create_error.message,
            resolved: create_error.resolved,
            namespace_id: create_error.namespace_id,
            stack_trace: create_error.stack_trace,
            user_affected: create_error.user_affected,
        })
    }

    pub async fn get_error_by_id(
        &self,
        id: Uuid,
    ) -> Result<ErrorDTO, ServerError> {
        let found_error = ErrorEntity::find()
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::Id.eq(id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::ErrorNotFound))?;
        
        let found_tags = TagEntity::find()
            .filter(<TagEntity as sea_orm::EntityTrait>::Column::ErrorId.eq(id))
            .all(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let tags = Some(found_tags.into_iter().map(|tag| ShortTagDTO {
            id: tag.id,
            tag_key: tag.tag_key,
            tag_value: tag.tag_value,
            tag_color: tag.tag_color,
        }).collect());

        Ok(ErrorDTO {
            id: found_error.id,
            user_affected: found_error.user_affected,
            message: found_error.message,
            path: found_error.path,
            line: found_error.line,
            stack_trace: found_error.stack_trace,
            namespace_id: found_error.namespace_id,
            resolved: found_error.resolved,
            created_at: found_error.created_at,
            updated_at: found_error.updated_at,
            tags,
        })
    }

    pub async fn update_error(
        &self,
        error: UpdateErrorDTO,
    ) -> Result<UpdateErrorDTO, ServerError> {
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

        let mut tag_list: Vec<TagDTO> = Vec::new();

        if let Some(tags) = error.tags {
            for tag in tags {
                let tag_clone = tag.clone();
                let tag_model = TagModel {
                    id: Uuid::new_v4(),
                    tag_key: tag_clone.tag_key,
                    tag_value: tag_clone.tag_value,
                    error_id: update_error.id,
                    tag_color: tag_clone.tag_color,
                }.into_active_model();

                tag_list.push(tag);

                TagEntity::insert(tag_model)
                    .exec(&*self.db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
            }
        }

        update_error.updated_at = now;

        ErrorEntity::update(update_error.clone().into_active_model())
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(UpdateErrorDTO {
            id: update_error.id,
            resolved: Some(update_error.resolved),
            tags: Some(tag_list)
        })
    }

    pub async fn get_aggregate_errors_by_date(
        &self,
        namespace_id: Uuid,
        selected_date: NaiveDate,
        time_interval: i64,
        timezone: String,
    ) -> Result<Vec<AggregateErrorDTO>, ServerError> {
        let user_tz: Tz = timezone.parse().map_err(|_| ServerError::QueryError(QueryError::InvalidTimestamp))?;

        let start_time_naive = selected_date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| ServerError::QueryError(QueryError::InvalidTimestamp))?;
        let start_time_local = user_tz.from_local_datetime(&start_time_naive)
            .single()
            .ok_or_else(|| ServerError::QueryError(QueryError::InvalidTimestamp))?;
        let end_time_local = start_time_local + Duration::days(1);

        // Convert local times to UTC
        let start_time_utc = start_time_local.with_timezone(&Utc);
        let end_time_utc = end_time_local.with_timezone(&Utc);

        let errors = ErrorEntity::find()
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt.gt(start_time_utc))
            .filter(<ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt.lt(end_time_utc))
            .all(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let mut error_map: HashMap<DateTime<Utc>, i64> = HashMap::new();

        for error in errors {
            let time_bucket_seconds = (error.created_at.timestamp() / (time_interval * 60)) * (time_interval * 60);
            let time_bucket = match Utc.timestamp_opt(time_bucket_seconds, 0) {
                chrono::LocalResult::Single(dt) => dt,
                _ => return Err(ServerError::QueryError(QueryError::InvalidTimestamp)),
            };

            *error_map.entry(time_bucket).or_insert(0) += 1;
        }

        let mut aggregated_errors: Vec<AggregateErrorDTO> = error_map
            .into_iter()
            .map(|(time, count)| AggregateErrorDTO { count, time })
            .collect();

        aggregated_errors.sort_by(|a, b| a.time.cmp(&b.time));

        Ok(aggregated_errors)
    }

    pub async fn get_error_metadata_by_group(
        &self,
        namespace_id: Uuid,
        group_by: String,
        group_key: String,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<ErrorMetaDTO>, ServerError> {
        let db: &DatabaseConnection = &*self.db;

        let errors = if group_by == "tags" {
            let parts: Vec<&str> = group_key.split(':').collect();
            if parts.len() != 2 {
                return Err(ServerError::QueryError(QueryError::InvalidTag));
            }
            let tag_key = parts[0].trim();
            let tag_value = parts[1].trim();

            ErrorEntity::find()
                .filter(<ErrorEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
                .filter(
                    Condition::any()
                        .add(<ErrorEntity as EntityTrait>::Column::Id.in_subquery(
                            Query::select()
                                .column(<TagEntity as EntityTrait>::Column::ErrorId)
                                .from(TagEntity)
                                .and_where(<TagEntity as EntityTrait>::Column::TagKey.eq(tag_key))
                                .and_where(<TagEntity as EntityTrait>::Column::TagValue.eq(tag_value))
                                .to_owned(),
                        )),
                )
                .order_by_desc(<ErrorEntity as EntityTrait>::Column::CreatedAt)
                .offset(offset as u64)
                .limit(limit as u64)
                .all(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
        } else {
            ErrorEntity::find()
                .filter(<ErrorEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
                .filter(match group_by.as_str() {
                    "message" => <ErrorEntity as EntityTrait>::Column::Message.eq(group_key),
                    _ => <ErrorEntity as EntityTrait>::Column::Message.eq(group_key),
                })
                .order_by_desc(<ErrorEntity as EntityTrait>::Column::CreatedAt)
                .offset(offset as u64)
                .limit(limit as u64)
                .all(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
        };
        
        let error_meta = errors.into_iter().map(|error| {
            ErrorMetaDTO {
                id: error.id,
                created_at: error.created_at,
                // user_agent: error.user_agent,
                resolved: error.resolved,
            }
        }).collect();

        Ok(error_meta)
    }
}
