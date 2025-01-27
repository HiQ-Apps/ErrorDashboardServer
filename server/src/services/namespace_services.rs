use chrono::Utc;
use bcrypt::{hash, DEFAULT_COST};
use futures::stream::{FuturesUnordered, TryStreamExt};
use sea_orm::{entity::prelude::*, ActiveValue, EntityTrait, IntoActiveModel, DatabaseConnection, QuerySelect, TransactionTrait};
use shared_types::user_dtos::MemberListDTO;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use log::info;

use shared_types::namespace_dtos::{GetNamespaceResponseDTO, GetNamespacesByUserResponseDTO, InviteUserRequestDTO, ShortNamespaceDTO, UpdateNamespaceDTO};
use shared_types::notification_dtos::NotificationDTO;
use shared_types::error_dtos::{AggregatedResult, GetAggregatedLineErrorDTO, AggregateIndividualErrorDTO, GetAggregatedMessageErrorDTO, GetAggregatedStatusErrorDTO, TagAggregatedErrorDTO};
use shared_types::tag_dtos::ShortTagNoIdDTO;
use crate::config::Config;
use crate::managers::notification_manager::{self, NotificationServer};
use crate::models::namespace_model::{Entity as NamespaceEntity, Model as NamespaceModel};
use crate::models::notification_model::{Entity as NotificationEntity, Model as NotificationModel};
use crate::models::user_model::Entity as UserEntity;
use crate::models::error_model::Entity as ErrorEntity;
use crate::models::error_tag_model::Entity as TagEntity;
use crate::models::user_namespace_junction_model::{Entity as UserNamespaceJunctionEntity, Model as UserNamespaceJunctionModel};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError, RequestError};
use crate::shared::utils::role::{get_perms, string_to_role, Permission, Role, RoleRules};


pub struct NamespaceService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>
}

impl NamespaceService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn get_all_namespaces(&self) -> Result<Vec<ShortNamespaceDTO>, ServerError> {
        let db = &*self.db;
        let namespaces = NamespaceEntity::find().all(db).await.map_err(ExternalError::from)?;

        let short_namespaces = namespaces.into_iter().map(|namespace| ShortNamespaceDTO {
            id: namespace.id,
            active: namespace.active,
            service_name: namespace.service_name,
            environment_type: namespace.environment_type,
        }).collect();

        Ok(short_namespaces)
    }

    pub async fn create_namespace(
        &self,
        user_id: Uuid,
        namespace_service_name: String,
        environment_type: String,
        notification_manager: Arc<NotificationServer>
    ) -> Result<Uuid, ServerError> {
        let db = &*self.db;
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let uid = Uuid::new_v4();
        let new_client_id = Uuid::new_v4();
        let now = Utc::now();
        let namespace_name_clone = namespace_service_name.clone();

        // Check if user has 10 > namespaces
        let user_namespace_count = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .count(db)
            .await.map_err(ExternalError::from)?;

        if user_namespace_count >= 10 {
            return Err(ServerError::RequestError(RequestError::NamespaceLimitReached));
        }

        let namespace = NamespaceModel {
            id: uid,
            active: true,
            service_name: namespace_service_name,
            environment_type,
            client_id: new_client_id,
            client_secret: Uuid::new_v4(),
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
            role: "owner".to_string(),
        }
        .into_active_model();

        if let Err(err) = UserNamespaceJunctionEntity::insert(user_namespace_junction).exec(&transaction).await {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::from(ExternalError::from(err)));
        }

        transaction.commit().await.map_err(ExternalError::from)?;

        let notification = NotificationDTO {
            id: Uuid::new_v4(),
            title: "New Namespace Created".to_string(),
            source: "Higuard Support".to_string(),
            user_id,
            text: format!("You have created a new namespace: {}. You can begin sending errors now. View to documentation to get started.", namespace_name_clone),
            is_read: false,
            created_at: now,
        };
        let broadcast_notification = notification.clone();
        let notification_model = NotificationModel::from(notification).into_active_model();
        
        NotificationEntity::insert(notification_model)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        notification_manager.broadcast_notification(broadcast_notification, &user_id).await;
        
        Ok(uid)
    }

    pub async fn get_namespace_by_id(&self, uid: Uuid) -> Result<GetNamespaceResponseDTO, ServerError> {
        let config = &*self.configs;
        let result = NamespaceEntity::find().filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.eq(uid)).one(&*self.db).await;
        let hash_cost = config.hash_cost.parse().unwrap_or(DEFAULT_COST);

        match result {
            Ok(Some(namespace)) => {
                let hash_secret = hash(namespace.client_secret.to_string(), hash_cost).unwrap();
                let namespace_dto = GetNamespaceResponseDTO {
                    id: namespace.id,
                    active: namespace.active,
                    service_name: namespace.service_name,
                    environment_type: namespace.environment_type,
                    client_id: namespace.client_id,
                    client_secret: hash_secret,
                    created_at: namespace.created_at,
                    updated_at: namespace.updated_at,
                };    
                Ok(namespace_dto)
            },
            Ok(None) => Err(ServerError::QueryError(QueryError::NamespaceNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err)))
        }
    }
    
    pub async fn get_namespaces_by_user_id(&self, user_id: Uuid, offset: u64, limit: u64) -> Result<Vec<GetNamespacesByUserResponseDTO>, ServerError> {
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

                    let short_namespaces = namespaces.into_iter().map(|namespace| GetNamespacesByUserResponseDTO {
                        id: namespace.id,
                        active: namespace.active,
                        service_name: namespace.service_name,
                        environment_type: namespace.environment_type,
                        role: junctions.iter().find(|junc| junc.namespace_id == namespace.id).unwrap().role.clone(),
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
        update_namespace_object: UpdateNamespaceDTO,
        role_rules: RoleRules) -> Result<(), ServerError> {
        let db: &DatabaseConnection = &*self.db;
        let config = &*self.configs;

        let transaction = db.begin().await.map_err(ExternalError::from)?;
        let now = Utc::now();

        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(update_namespace_object.id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .one(&transaction)
            .await;

        let namespace_junc = match namespace_junc_result {
            Ok(Some(junction)) => {
                let role_permissions = get_perms(&junction.role, &role_rules);
                if let Some(role_permissions) = role_permissions {
                    if !role_permissions.permissions.contains(&Permission::Update) {
                        transaction.rollback().await.map_err(ExternalError::from)?;
                        return Err(ServerError::RequestError(RequestError::PermissionDenied));
                    } else {
                        junction
                    }
                } else {
                    transaction.rollback().await.map_err(ExternalError::from)?;
                    return Err(ServerError::RequestError(RequestError::PermissionDenied));
                }
            },
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
            if client_secret == true {
                namespace.client_secret = ActiveValue::Set(Uuid::new_v4());
            };
        };

        namespace.updated_at = ActiveValue::Set(now);

        match namespace.update(&transaction).await {
            Ok(_) => (),
            Err(err) => {
                transaction.rollback().await.map_err(ExternalError::from)?;
                return Err(ServerError::from(ExternalError::from(err)));
            }
        };

        transaction.commit().await.map_err(ExternalError::from)?;
        Ok(())
    }

    pub async fn delete_namespace(&self, 
        namespace_id: Uuid, 
        user_id: Uuid, 
        notification_manager: Arc<NotificationServer>
    ) -> Result<(), ServerError> {
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
            return Err(ServerError::from(ExternalError::DB(err)));
        },
    };

    if namespace_junc.user_id != user_id {
        transaction.rollback().await.map_err(ExternalError::from)?;
        return Err(ServerError::RequestError(RequestError::PermissionDenied));
    };

    // Alert all users that namespace is being deleted
    let subscribed_users = UserNamespaceJunctionEntity::find()
        .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
        .all(&transaction)
        .await
        .map_err(ExternalError::from)?;

    for user in subscribed_users {
        let notification = NotificationDTO {
            id: Uuid::new_v4(),
            title: "Namespace Deleted".to_string(),
            source: "Higuard Support".to_string(),
            user_id: user.user_id,
            text: format!("The namespace {} has been deleted. You will no longer receive alerts for this namespace.", namespace.service_name),
            is_read: false,
            created_at: Utc::now(),
        };
        let notification_clone = notification.clone();
        let notification_model = NotificationModel::from(notification).into_active_model();
        
        NotificationEntity::insert(notification_model)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        
        notification_manager.broadcast_notification(notification_clone, &user.user_id).await;
    }

    if let Err(err) = namespace.delete(&transaction).await {
        transaction.rollback().await.map_err(ExternalError::from)?;
        return Err(ServerError::from(ExternalError::DB(err)));
    };


    transaction.commit().await.map_err(ExternalError::from)?;

    Ok(())
}

    pub async fn get_errors_by_namespace_id(
        &self,
        namespace_id: Uuid,
    ) -> Result<Vec<AggregateIndividualErrorDTO>, ServerError> {
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
                    tag_color: tag.tag_color
                }).collect::<Vec<ShortTagNoIdDTO>>());

                Ok(AggregateIndividualErrorDTO {
                    id: error.id,
                    user_affected: error.user_affected,
                    message: error.message.clone(),
                    line: error.line,
                    path: error.path,
                    stack_trace: error.stack_trace,
                    namespace_id: error.namespace_id,
                    resolved: error.resolved,
                    created_at: error.created_at,
                    tags,
                    updated_at: error.updated_at,
                }) as Result<AggregateIndividualErrorDTO, ServerError>
            }
        }).collect::<FuturesUnordered<_>>();

        let mut results: Vec<AggregateIndividualErrorDTO> = futures.try_collect().await?;
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }

    async fn aggregate_errors(
        &self,
        errors: &[AggregateIndividualErrorDTO],
        group_by: &str,
    ) -> (
        HashMap<String, GetAggregatedMessageErrorDTO>,
        HashMap<String, GetAggregatedLineErrorDTO>,
        HashMap<ShortTagNoIdDTO, TagAggregatedErrorDTO>,
        HashMap<String, HashSet<String>>,
    ) {
        let mut grouped_errors_by_message: HashMap<String, GetAggregatedMessageErrorDTO> = HashMap::new();
        let mut grouped_errors_by_line: HashMap<String, GetAggregatedLineErrorDTO> = HashMap::new();
        let mut unique_users: HashMap<String, HashSet<String>> = HashMap::new();
        let mut tag_aggregations: HashMap<ShortTagNoIdDTO, TagAggregatedErrorDTO> = HashMap::new();

        for error in errors {
            if group_by == "tags" {
                self.aggregate_by_tags(&mut tag_aggregations, &mut unique_users, error);
            } else {
                self.aggregate_by_other(
                    &mut grouped_errors_by_message,
                    &mut grouped_errors_by_line,
                    &mut unique_users,
                    error,
                    group_by,
                );
            }
        }
        (
            grouped_errors_by_message,
            grouped_errors_by_line,
            tag_aggregations,
            unique_users,
        )
    }

    fn aggregate_by_tags(
        &self,
        tag_aggregations: &mut HashMap<ShortTagNoIdDTO, TagAggregatedErrorDTO>,
        unique_users: &mut HashMap<String, HashSet<String>>,
        error: &AggregateIndividualErrorDTO,
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
        grouped_errors_by_message: &mut HashMap<String, GetAggregatedMessageErrorDTO>,
        grouped_errors_by_line: &mut HashMap<String, GetAggregatedLineErrorDTO>,
        unique_users: &mut HashMap<String, HashSet<String>>,
        error: &AggregateIndividualErrorDTO,
        group_by: &str,
    ) {
        match group_by {
            "message" => {
                let key = error.message.clone();
                let entry = grouped_errors_by_message.entry(key.clone()).or_insert_with(|| GetAggregatedMessageErrorDTO {
                    message: error.message.clone(),
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
                    let combined_tags: HashSet<ShortTagNoIdDTO> = tag_set.union(&new_tags).cloned().collect();
                    entry.aggregated_tags = combined_tags.into_iter().collect();
                }
            }
            "line" => {
                let key = error.line.clone();
                let entry = grouped_errors_by_line.entry(key.to_string()).or_insert_with(|| GetAggregatedLineErrorDTO {
                    line: error.line.clone(),
                    user_affected_count: 0,
                    error_count: 0,
                    aggregated_tags: vec![],
                });

                entry.error_count += 1;

                let user_set = unique_users.entry(key.to_string()).or_insert_with(HashSet::new);
                user_set.insert(error.user_affected.clone());
                entry.user_affected_count = user_set.len() as i32;

                if let Some(tags) = &error.tags {
                    let tag_set: HashSet<ShortTagNoIdDTO> = entry.aggregated_tags.iter().cloned().collect();
                    let new_tags: HashSet<ShortTagNoIdDTO> = tags.iter().cloned().collect();
                    let combined_tags: HashSet<ShortTagNoIdDTO> = tag_set.union(&new_tags).cloned().collect();
                    entry.aggregated_tags = combined_tags.into_iter().collect();
                }
            }
            _ => {}
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
        let (
            grouped_errors_by_message,
            // grouped_errors_by_status,
            grouped_errors_by_line,
            tag_aggregations,
            _
        ) = self.aggregate_errors(&errors, &group_by).await;

        match group_by.as_str() {
            "tags" => {
                let mut tag_results: Vec<TagAggregatedErrorDTO> = tag_aggregations.into_iter().map(|(_, v)| v).collect();
                tag_results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
                let paginated_results = tag_results.into_iter().skip(offset).take(limit).collect();
                Ok(AggregatedResult::ByTags(paginated_results))
            }
            "message" => {
                let mut message_results: Vec<GetAggregatedMessageErrorDTO> = grouped_errors_by_message
                    .into_iter()
                    .map(|(_, v)| v)
                    .collect();
                message_results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
                let paginated_results = message_results.into_iter().skip(offset).take(limit).collect();
                Ok(AggregatedResult::ByMessage(paginated_results))
            }
            "line" => {
                let mut line_results: Vec<GetAggregatedLineErrorDTO> = grouped_errors_by_line
                    .into_iter()
                    .map(|(_, v)| v)
                    .collect();
                line_results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
                let paginated_results = line_results.into_iter().skip(offset).take(limit).collect();
                Ok(AggregatedResult::ByLine(paginated_results))
            }
            _ => {
                let mut message_results: Vec<GetAggregatedMessageErrorDTO> = grouped_errors_by_message
                    .into_iter()
                    .map(|(_, v)| v)
                    .collect();
                message_results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
                let paginated_results = message_results.into_iter().skip(offset).take(limit).collect();
                Ok(AggregatedResult::ByMessage(paginated_results))
            }
        }
    }

    pub async fn invite_user_to_namespace(&self, namespace_id: Uuid, invite_data: InviteUserRequestDTO) -> Result<(), ServerError> {
        let db = &*self.db;

        let user_namespace_junction = UserNamespaceJunctionModel {
            id: Uuid::new_v4(),
            user_id: invite_data.user_id,
            namespace_id,
            role: invite_data.role,
        };

        if let Err(err) = UserNamespaceJunctionEntity::insert(user_namespace_junction.into_active_model()).exec(db).await {
            return Err(ServerError::from(ExternalError::from(err)));
        }

        Ok(())
    }

    pub async fn remove_user_from_namespace(&self, user_id: Uuid, namespace_id: Uuid) -> Result<(), ServerError> {
        let db = &*self.db;

        let found_instance = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;

        let found_instance = match found_instance {
            Ok(Some(instance)) => instance,
            Ok(None) => {
                return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound,
                ));
            }
            Err(err) => return Err(ServerError::from(ExternalError::from(err))),
        };

        found_instance.delete(db).await.map_err(ExternalError::from)?;
        Ok(())
    }

    pub async fn check_namespace_membership(&self, user_id: Uuid, namespace_id: Uuid) -> Result<bool, ServerError>{
        let db = &*self.db;
        let result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;

        match result {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(err) => Err(ServerError::from(ExternalError::from(err))),
        }
    }

    // Does the user have the required permissions to do a specific action in a namespace?
    pub async fn check_user_namespace_perms(&self, user_id: Uuid, namespace_id: Uuid, rules: &RoleRules, action: Permission) -> Result<bool, ServerError> {
        let db = &*self.db;
        let result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;
        
        match result {
            Ok(Some(junction)) => {
                let role = junction.role;
                let perms = string_to_role(&role);

                match perms {
                    Some(role) => {
                        let role_perms = rules.role_rules.get(&role).unwrap();
                        if role_perms.permissions.contains(&action) {
                            return Ok(true);
                        }
                        return Err(ServerError::RequestError(RequestError::PermissionDenied));
                    },
                    None => return Err(ServerError::RequestError(RequestError::PermissionDenied)),
                };
            },
            Ok(None) => Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err))),
        }

    }

    pub async fn compare_user_namespace_perms(&self, requester_id: Uuid, target_id: Uuid, namespace_id: Uuid, rules: &RoleRules ) -> Result<bool, ServerError> {
        let db = &*self.db;
        let requester_junction = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(requester_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;

        let target_junction = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(target_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;

        match (requester_junction, target_junction) {
            (Ok(Some(requester)), Ok(Some(target))) => {
                let requester_role = requester.role;
                let target_role = target.role;

                let requester_perms = string_to_role(&requester_role);
                let target_perms = string_to_role(&target_role);

                match (requester_perms, target_perms) {
                    (Some(requester_role), Some(target_role)) => {
                        let requester_weight = rules.role_rules.get(&requester_role).unwrap().weight;
                        let target_weight = rules.role_rules.get(&target_role).unwrap().weight;

                        if requester_weight >= target_weight {
                            return Ok(true);
                        }
                        return Err(ServerError::RequestError(RequestError::PermissionDenied));
                    },
                    _ => return Err(ServerError::RequestError(RequestError::PermissionDenied)),
                }
            },
            _ => return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound)),
        }

    }

    pub async fn get_namespace_members(&self, namespace_id: Uuid) -> Result<Vec<MemberListDTO>, ServerError> {
        let db = &*self.db;
        let junctions = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .all(db)
            .await
            .map_err(ExternalError::from)?;

        let user_ids: Vec<Uuid> = junctions.iter().map(|junc| junc.user_id).collect();
        let users = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Id.is_in(user_ids))
            .all(db)
            .await
            .map_err(ExternalError::from)?;

        let user_namespace_junction = junctions.into_iter().map(|junc| (junc.user_id, junc.role)).collect::<HashMap<Uuid, String>>();
        
        let users_dto: Vec<MemberListDTO> = users.into_iter().map(|user| {
            MemberListDTO {
                id: user.id,
                email: user.email,
                username: user.username,
                role: user_namespace_junction.get(&user.id)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
            }
        }).collect();

        Ok(users_dto)
    }

    pub async fn get_user_role_in_namespace(&self, user_id: Uuid, namespace_id: Uuid) -> Result<String, ServerError> {
        let db = &*self.db;
        let result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;

        match result {
            Ok(Some(junction)) => Ok(junction.role),
            Ok(None) => Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::from(err))),
        }
    }

    pub async fn update_user_role_in_namespace(&self, user_id: Uuid, namespace_id: Uuid, new_role: String) -> Result<(), ServerError> {
        let db = &*self.db;
        let result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(db)
            .await;

        let junction = match result {
            Ok(Some(junction)) => junction,
            Ok(None) => return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound)),
            Err(err) => return Err(ServerError::from(ExternalError::from(err))),
        };

        let mut junction_active = junction.into_active_model();
        junction_active.role = ActiveValue::Set(new_role);

        junction_active.update(db).await.map_err(ExternalError::from)?;
        Ok(())
    }
}
