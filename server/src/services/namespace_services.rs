use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue, EntityTrait, IntoActiveModel, DatabaseConnection, TransactionError, TransactionTrait};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::namespace_dtos::{NamespaceDto, UpdateNamespaceDto};
use crate::config::Config;
use crate::models::namespace_model::{Entity as NamespaceEntity, Model as NamespaceModel};
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
            transaction.rollback().await.map_err(|rollback_err| {
                ServerError::ExternalError(ExternalError::DB(rollback_err))
            })?;
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }

        let user_namespace_junction = UserNamespaceJunctionModel {
            id: Uuid::new_v4(),
            user_id,
            namespace_id: uid,
        }
        .into_active_model();

        if let Err(err) = UserNamespaceJunctionEntity::insert(user_namespace_junction).exec(&transaction).await {
            transaction.rollback().await.map_err(|rollback_err| {
                ServerError::ExternalError(ExternalError::DB(rollback_err))
            })?;
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }

        transaction.commit().await.map_err(|err| {
            ServerError::ExternalError(ExternalError::DB(err))
        })?;

        Ok(uid)
    }

    pub async fn get_namespace_by_id(&self, uid: Uuid) -> Result<NamespaceModel, ServerError> {
        let result = NamespaceEntity::find().filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.eq(uid)).one(&*self.db).await;
        
        match result {
            Ok(Some(namespace)) => Ok(namespace),
            Ok(None) => Err(ServerError::QueryError(QueryError::NamespaceNotFound)),
            Err(err) => Err(ServerError::from(ExternalError::DB(err))),
        }
    }
    
    pub async fn get_namespaces_by_user_id(&self, user_id: Uuid) -> Result<Vec<NamespaceModel>, ServerError> {
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
                        .all(&*self.db)
                        .await
                        .map_err(|err| ServerError::from(ExternalError::DB(err)))?;
                    Ok(namespaces)
                }
            },
            Err(err) => {
                Err(ServerError::from(ExternalError::DB(err)))
            }
        }
    }
    
    pub async fn update_namespace(&self, update_namespace_object: UpdateNamespaceDto, current_user_id: Uuid) -> Result<NamespaceDto, ServerError> {
        let db: &DatabaseConnection = &*self.db;
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        let now = Utc::now();

        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::Id.eq(update_namespace_object.id))
            .one(&transaction)
            .await;

        let namespace_junc = match namespace_junc_result {
            Ok(Some(junction)) => junction,
            Ok(None) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::from(ExternalError::DB(err)));
            },
        };

        if namespace_junc.user_id != current_user_id {
            transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
            return Err(ServerError::RequestError(RequestError::PermissionDenied));
        };

        let namespace_result = NamespaceEntity::find()
            .filter(<NamespaceEntity as EntityTrait>::Column::Id.eq(namespace_junc.namespace_id))
            .one(&transaction)
            .await;

        let mut namespace = match namespace_result {
            Ok(Some(namespace)) => namespace.into_active_model(),
            Ok(None) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::QueryError(QueryError::NamespaceNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
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

        let updated_namespace = match namespace.update(&transaction).await {
            Ok(namespace) => namespace,
            Err(err) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::from(ExternalError::DB(err)));
            }
        };

        transaction.commit().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let updated_namespace_dto = NamespaceDto {
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
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .one(&transaction)
            .await;

        let namespace_junc = match namespace_junc_result {
            Ok(Some(junction)) => junction,
            Ok(None) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
                return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound));
            },
            Err(err) => {
                transaction.rollback().await.map_err(|rollback_err| ServerError::ExternalError(ExternalError::DB(rollback_err)))?;
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

        if let Err(err) = namespace_junc.delete(&transaction).await {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::from(ExternalError::from(err)));
        };

        transaction.commit().await.map_err(ExternalError::from)?;

        Ok(())
    }

}