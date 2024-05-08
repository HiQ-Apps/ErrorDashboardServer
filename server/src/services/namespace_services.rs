use actix_web::HttpResponse;
use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue, Set, EntityTrait, IntoActiveModel};
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
        environment_type: String
    ) -> Result<Uuid, ServerError> {
        let uid = Uuid::new_v4();
        let new_client_secret = Uuid::new_v4();
        let new_client_id = Uuid::new_v4();
        let now = Utc::now();

        let namespace = NamespaceModel {
            id: uid,
            active: false,
            service_name: namespace_service_name,
            environment_type,
            client_id: new_client_secret,
            client_secret: new_client_id,
            error_records: None,
            created_at: now,
            updated_at: now,
        }.into_active_model();

        let namespace_query = NamespaceEntity::insert(namespace).exec(&*self.db).await;
        
        let user_namespace_junction = UserNamespaceJunctionModel {
            id: Uuid::new_v4(),
            user_id,
            namespace_id: uid,
        }.into_active_model();
        
        UserNamespaceJunctionEntity::insert(user_namespace_junction).exec(&*self.db).await;

        match namespace_query {
            Ok(_) => Ok(uid),
            Err(err) => Err(ServerError::from(ExternalError::DB(err)))
        }
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
    
    pub async fn update_namespace(&self, update_namespace_object: UpdateNamespaceDto, current_user_id: Uuid) -> Result<HttpResponse, ServerError> {
        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::Id.eq(update_namespace_object.id))
            .one(&*self.db)
            .await;

        let namespace_junc = match namespace_junc_result {
            Ok(Some(junction)) => junction,
            Ok(None) => return Err(ServerError::QueryError(QueryError::UserNamespaceJunctionNotFound)),
            Err(err) => return Err(ServerError::from(ExternalError::DB(err))),
        };

        if namespace_junc.user_id != current_user_id {
            return Err(ServerError::RequestError(RequestError::PermissionDenied));
        };

        let namespace_result = NamespaceEntity::find()
            .filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.eq(namespace_junc.namespace_id))
            .one(&*self.db)
            .await;

        let mut namespace = match namespace_result {
            Ok(Some(namespace)) => namespace.into_active_model(),
            Ok(None) => return Err(ServerError::QueryError(QueryError::NamespaceNotFound)),
            Err(err) => return Err(ServerError::from(ExternalError::DB(err))),
        };

        if let Some(active) = update_namespace_object.active {
            namespace.active = ActiveValue::Set(active);
        };

        if let Some(name) = update_namespace_object.service_name {
            namespace.service_name = ActiveValue::Set(name);
        };

        namespace.update(&*self.db).await.map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        Ok(HttpResponse::Ok().finish())
    }

    
    pub async fn delete_namespace(&self, user_id: Uuid, namespace_id: Uuid) -> Result<HttpResponse, ServerError> {
        let found_namespace_junc = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        let namespace_junc = match found_namespace_junc {
            Some(junction) => junction,
            None => return Err(ServerError::from(RequestError::PermissionDenied)),
        };
        
        let namespace_junc_active_model = namespace_junc.into_active_model();

        let delete_namespace_result = NamespaceEntity::delete_by_id(namespace_id)
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        if delete_namespace_result.rows_affected == 0 {
            return Err(ServerError::from(QueryError::NamespaceNotFound));
        }

        // Delete junction too
        let delete_junction_result = UserNamespaceJunctionEntity::delete(namespace_junc_active_model)
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

        if delete_junction_result.rows_affected == 0 {
            // Log this as it's unusual if we've just deleted the namespace but not the junction
            return Err(ServerError::from(QueryError::UserNamespaceJunctionNotFound));
        }

        Ok(HttpResponse::Ok().finish())
    }
}
