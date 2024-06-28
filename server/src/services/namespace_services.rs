use chrono::Utc;
use futures::stream::{FuturesUnordered, TryStreamExt};
use sea_orm::{entity::prelude::*, ActiveValue, EntityTrait, IntoActiveModel, QueryOrder, DatabaseConnection, QuerySelect, TransactionTrait};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use log::info;

use shared_types::namespace_dtos::{NamespaceDTO, UpdateNamespaceDTO, ShortNamespaceDTO};
use shared_types::error_dtos::{AggregatedResult, ErrorDTO, ErrorMetaDTO, GetAggregatedErrorDTO, TagAggregatedErrorDTO};
use shared_types::tag_dtos::ShortTagNoIdDTO;
use crate::config::Config;
use crate::models::namespace_model::{Entity as NamespaceEntity, Model as NamespaceModel};
use crate::models::error_model::Entity as ErrorEntity;
use crate::models::error_tag_model::Entity as TagEntity;
use crate::models::user_namespace_junction_model::{Entity as UserNamespaceJunctionEntity, Model as UserNamespaceJunctionModel};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError, RequestError};


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
        user_id: Uuid,
        namespace_service_name: String,
        environment_type: String,
    ) -> Result<Uuid, ServerError> {
        let db = &*self.db;
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let uid = Uuid::new_v4();
        let new_client_secret = Uuid::new_v4();
        let new_client_id = Uuid::new_v4();
        let now = Utc::now();

        let namespace = NamespaceModel {
            id: uid,
            active: true,
            service_name: namespace_service_name,
            environment_type,
            client_id: new_client_id,
            client_secret: new_client_secret,
            created_at: now,
            updated_at: now,
        }
        .into_active_model();

        if let Err(err) = NamespaceEntity::insert(namespace).exec(&transaction).await {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::from(ExternalError::from(err)));
        }

        let user_namespace_junction = UserNamespaceJunctionModel {
            id: Uuid::new_v4(),
            user_id,
            namespace_id: uid,
        }
        .into_active_model();

        if let Err(err) = UserNamespaceJunctionEntity::insert(user_namespace_junction).exec(&transaction).await {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::from(ExternalError::from(err)));
        }

        transaction.commit().await.map_err(ExternalError::from)?;

        Ok(uid)
    }

    pub async fn get_namespace_by_id(&self, uid: Uuid) -> Result<NamespaceModel, ServerError> {
        let result = NamespaceEntity::find().filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.eq(uid)).one(&*self.db).await;
        
        match result {
            Ok(Some(namespace)) => Ok(namespace),
            Ok(None) => Err(ServerError::QueryError(QueryError::NamespaceNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err)))
        }
    }
    
    pub async fn get_namespaces_by_user_id(&self, user_id: Uuid, offset: u64, limit: u64) -> Result<Vec<ShortNamespaceDTO>, ServerError> {
        let junction_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::UserId.eq(user_id))
            .all(&*self.db)
            .await;

        match junction_result {
            Ok(junctions) => {
                if junctions.is_empty() {
                    Ok(vec![])
                } else {
                    let namespace_ids: Vec<Uuid> = junctions.iter().map(|junc| junc.namespace_id).collect();
                    let namespaces = NamespaceEntity::find()
                        .filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.is_in(namespace_ids))
                        .offset(offset)
                        .limit(limit)
                        .all(&*self.db)
                        .await
                        .map_err(ExternalError::from)?;

                    let short_namespaces = namespaces.into_iter().map(|namespace| ShortNamespaceDTO {
                        id: namespace.id,
                        active: namespace.active,
                        service_name: namespace.service_name,
                        environment_type: namespace.environment_type,
                    }).collect();

                    Ok(short_namespaces)
                }
            },
            Err(err) => {
                Err(ServerError::from(ExternalError::from(err)))
            }
        }
    }
    
    pub async fn update_namespace(
        &self,
        user_id: Uuid,
        update_namespace_object: UpdateNamespaceDTO) -> Result<NamespaceDTO, ServerError> {
        let db: &DatabaseConnection = &*self.db;
        let transaction = db.begin().await.map_err(ExternalError::from)?;
        let now = Utc::now();

        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(update_namespace_object.id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .one(&transaction)
            .await;

        let namespace_junc = match namespace_junc_result {
            Ok(Some(junction)) => junction,
            Ok(None) => {
                info!("UserNamespaceJunction not found. Rolling back transaction.");
                transaction.rollback().await.map_err(|err| ServerError::from(ExternalError::from(err)))?;
                return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(|err| ServerError::from(ExternalError::from(err)))?;
                return Err(ServerError::from(ExternalError::from(err)));
            },
        };

        if namespace_junc.user_id != user_id {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::RequestError(RequestError::PermissionDenied));
        };

        let namespace_result = NamespaceEntity::find()
            .filter(<NamespaceEntity as EntityTrait>::Column::Id.eq(namespace_junc.namespace_id))
            .one(&transaction)
            .await;

        let mut namespace = match namespace_result {
            Ok(Some(namespace)) => namespace.into_active_model(),
            Ok(None) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::QueryError(QueryError::NamespaceNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::from(ExternalError::DB(err)));
            },
        };

        if let Some(active) = update_namespace_object.active {
            namespace.active = ActiveValue::Set(active);
        };

        if let Some(name) = update_namespace_object.service_name {
            namespace.service_name = ActiveValue::Set(name);
        };

        if let Some(env_type) = update_namespace_object.environment_type {
            namespace.environment_type = ActiveValue::Set(env_type);
        };

        if let Some(client_id) = update_namespace_object.client_id {
            namespace.client_id = ActiveValue::Set(client_id);
        };

        if let Some(client_secret) = update_namespace_object.client_secret {
            namespace.client_secret = ActiveValue::Set(client_secret);
        };

        namespace.updated_at = ActiveValue::Set(now);

        let updated_namespace = match namespace.update(&transaction).await {
            Ok(namespace) => namespace,
            Err(err) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::from(ExternalError::from(err)));
            }
        };

        transaction.commit().await.map_err(ExternalError::from)?;

        let updated_namespace_dto = NamespaceDTO {
            id: updated_namespace.id,
            active: updated_namespace.active,
            service_name: updated_namespace.service_name,
            environment_type: updated_namespace.environment_type,
            client_id: updated_namespace.client_id,
            client_secret: updated_namespace.client_secret,
            created_at: updated_namespace.created_at,
            updated_at: now,
        };
        Ok(updated_namespace_dto)
    }

    pub async fn delete_namespace(&self, namespace_id: Uuid, user_id: Uuid) -> Result<(), ServerError> {
        let db: &DatabaseConnection = &*self.db;
        let transaction = db.begin().await.map_err(ExternalError::from)?;
        

        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .one(&transaction)
            .await;

        let namespace_junc = match namespace_junc_result {
            Ok(Some(junction)) => junction,
            Ok(None) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::from(ExternalError::DB(err)));
            },
        };

        let namespace_result = NamespaceEntity::find()
            .filter(<NamespaceEntity as EntityTrait>::Column::Id.eq(namespace_id))
            .one(&transaction)
            .await;

        let namespace = match namespace_result {
            Ok(Some(namespace)) => namespace,
            Ok(None) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::QueryError(QueryError::NamespaceNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::from(ExternalError::from(err)));
            },
        };

        if namespace_junc.user_id != user_id {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::RequestError(RequestError::PermissionDenied));
        };

        if let Err(err) = namespace.delete(&transaction).await {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::from(ExternalError::from(err)));
        };

        transaction.commit().await.map_err(ExternalError::from)?;

        Ok(())
    }

    pub async fn get_errors_by_namespace_id(
        &self,
        namespace_id: Uuid,
    ) -> Result<Vec<ErrorDTO>, ServerError> {
        let db: &DatabaseConnection = &*self.db;
        let errors = ErrorEntity::find()
            .filter(<ErrorEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let futures = errors.into_iter().map(|error| {
            let db = db.clone();
            async move {
                let error_id = error.id;
                let tags = TagEntity::find()
                    .filter(<TagEntity as EntityTrait>::Column::ErrorId.eq(error_id))
                    .all(&db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

                let tags = Some(tags.into_iter().map(|tag| ShortTagNoIdDTO {
                    tag_key: tag.tag_key,
                    tag_value: tag.tag_value,
                }).collect::<Vec<ShortTagNoIdDTO>>());

                Ok(ErrorDTO {
                    id: error.id,
                    status_code: error.status_code,
                    user_affected: error.user_affected,
                    path: error.path,
                    line: error.line,
                    message: error.message.clone(),
                    stack_trace: error.stack_trace,
                    namespace_id: error.namespace_id,
                    resolved: error.resolved,
                    created_at: error.created_at,
                    tags,
                    updated_at: error.updated_at,
                }) as Result<ErrorDTO, ServerError>
            }
        }).collect::<FuturesUnordered<_>>();

        let mut results: Vec<ErrorDTO> = futures.try_collect().await?;
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }

    pub fn match_group_by_errors(&self, group_by: &str, error: &ErrorDTO) -> String {
        match group_by {
            "status_code" => error.status_code.to_string(),
            "message" => error.message.clone(),
            "line" => error.line.to_string(),
            _ => "message".to_string(),
        }
    }

    async fn aggregate_errors(
        &self,
        errors: &[ErrorDTO],
        group_by: &str,
    ) -> (
        HashMap<String, GetAggregatedErrorDTO>,
        HashMap<ShortTagNoIdDTO, TagAggregatedErrorDTO>,
        HashMap<String, HashSet<String>>,
    ) {
        let mut grouped_errors: HashMap<String, GetAggregatedErrorDTO> = HashMap::new();
        let mut unique_users: HashMap<String, HashSet<String>> = HashMap::new();
        let mut tag_aggregations: HashMap<ShortTagNoIdDTO, TagAggregatedErrorDTO> = HashMap::new();

        for error in errors {
            if group_by == "tags" {
                self.aggregate_by_tags(&mut tag_aggregations, &mut unique_users, error);
            } else {
                self.aggregate_by_other(&mut grouped_errors, &mut unique_users, error, group_by);
            }
        }

        (grouped_errors, tag_aggregations, unique_users)
    }

    fn aggregate_by_tags(
        &self,
        tag_aggregations: &mut HashMap<ShortTagNoIdDTO, TagAggregatedErrorDTO>,
        unique_users: &mut HashMap<String, HashSet<String>>,
        error: &ErrorDTO,
    ) {
        if let Some(tags) = &error.tags {
            for tag in tags {
                let entry = tag_aggregations.entry(tag.clone()).or_insert_with(|| TagAggregatedErrorDTO {
                    tag: tag.clone(),
                    user_affected_count: 0,
                    error_count: 0,
                });

                entry.error_count += 1;

                let user_set = unique_users.entry(format!("{}:{}", tag.tag_key, tag.tag_value)).or_insert_with(HashSet::new);
                user_set.insert(error.user_affected.clone());
                entry.user_affected_count = user_set.len() as i32;
            }
        }
    }

    fn aggregate_by_other(
        &self,
        grouped_errors: &mut HashMap<String, GetAggregatedErrorDTO>,
        unique_users: &mut HashMap<String, HashSet<String>>,
        error: &ErrorDTO,
        group_by: &str,
    ) {
        let key = self.match_group_by_errors(group_by, error);

        let entry = grouped_errors.entry(key.clone()).or_insert_with(|| GetAggregatedErrorDTO {
            message: error.message.clone(),
            status_code: error.status_code,
            user_affected_count: 0,
            error_count: 0,
            aggregated_tags: vec![],
        });

        entry.error_count += 1;

        let user_set = unique_users.entry(key.clone()).or_insert_with(HashSet::new);
        user_set.insert(error.user_affected.clone());
        entry.user_affected_count = user_set.len() as i32;

        if let Some(tags) = &error.tags {
            let tag_set: HashSet<ShortTagNoIdDTO> = entry.aggregated_tags.iter().cloned().collect();
            let new_tags: HashSet<ShortTagNoIdDTO> = tags.iter().cloned().collect();
            let combined_tags = tag_set.union(&new_tags).cloned().collect();
            entry.aggregated_tags = combined_tags;
        }
    }

    pub async fn get_errors_by_namespace_with_pagination(
        &self,
        namespace_id: Uuid,
        group_by: String,
        offset: usize,
        limit: usize,
    ) -> Result<AggregatedResult, ServerError> {
        let errors = self.get_errors_by_namespace_id(namespace_id).await?;
        let (grouped_errors, tag_aggregations, _) = self.aggregate_errors(&errors, &group_by).await;

        match group_by.as_str() {
            "tags" => {
                let mut tag_results: Vec<TagAggregatedErrorDTO> = tag_aggregations.into_iter().map(|(_, v)| v).collect();
                tag_results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
                let paginated_results = tag_results.into_iter().skip(offset).take(limit).collect();
                Ok(AggregatedResult::ByTags(paginated_results))
            },
            _ => {
                let mut other_results: Vec<GetAggregatedErrorDTO> = grouped_errors.into_iter().map(|(_, v)| v).collect();
                other_results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
                let paginated_results = other_results.into_iter().skip(offset).take(limit).collect();
                Ok(AggregatedResult::ByOther(paginated_results))
            },
        }
    }
}
