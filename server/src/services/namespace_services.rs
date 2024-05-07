use actix_web::{http::StatusCode, HttpResponse};
use bcrypt::{verify, hash};
use chrono::Utc;
use sea_orm::{entity::prelude::*, Set, EntityTrait, IntoActiveModel};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::namespace_dtos::{NamespaceDto, UpdateNamespaceDto};
use crate::config::Config;
use crate::models::namespace_model::{Entity as NamespaceEntity, Model as NamespaceModel};
use crate::models::user_model::{Entity as UserEntity, Model as UserModel};
use crate::models::user_namespace_junction_model::{Entity as UserNamespaceJunctionEntity, Model as UserNamespaceJunctionModel};
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

        NamespaceEntity::insert(namespace).exec(&*self.db).await?;
        
        Ok(uid)
    }

    pub async fn get_namespace_by_id(&self, uid: Uuid) -> Result<NamespaceModel, ServerError> {
        let result = NamespaceEntity::find().filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.eq(uid)).one(&*self.db).await;
        
        match result {
            Ok(Some(namespace)) =>
                Ok(namespace),
            Ok(None) =>
                Err(ServerError::NamespaceNotFound),
            Err(err) => {
                Err(ServerError::DBError(err))
            }
        }
    }
    
    pub async fn get_namespaces_by_user_id(&self, user_id: Uuid) -> Result<Vec<NamespaceModel>, ServerError> {
        let junctions = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::UserId.eq(user_id))
            .all(&*self.db)
            .await
            .map_err(|err| ServerError::DBError(err))?;

        if junctions.is_empty() {
            Ok(vec![])
        } else {
            let namespace_ids: Vec<Uuid> = junctions.iter().map(|junc| junc.namespace_id).collect();
            let namespaces = NamespaceEntity::find()
                .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::Id.is_in(namespace_ids))
                .all(&*self.db)
                .await
                .map_err(|err| ServerError::DBError(err))?;

            Ok(namespaces)
        }
    }
    
    pub async fn update_namespace(&self, update_namespace_object: UpdateNamespaceDto, current_user_id: Uuid) -> Result<HttpResponse, ServerError> {
        let namespace_junc_result = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::Id.eq(update_namespace_object.id))
            .one(&*self.db)
            .await?;

        let namespace_junc = namespace_junc_result.ok_or(ServerError::NamespaceNotFound)?;

        if namespace_junc.user_id != current_user_id {
            return Err(ServerError::PermissionDenied);
        }

        let namespace_result = NamespaceEntity::find()
            .filter(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id.eq(namespace_junc.namespace_id))
            .one(&*self.db)
            .await?;

        let mut namespace = namespace_result.ok_or(ServerError::NamespaceNotFound)?.into_active_model();

        if let Some(active) = update_namespace_object.active {
            namespace.active = Set(active);
        }
        if let Some(name) = update_namespace_object.service_name {
            namespace.service_name = Set(name);
        }
        if let Some(env_type) = update_namespace_object.environment_type {
            namespace.environment_type = Set(env_type);
        }
        if let Some(client_id) = update_namespace_object.client_id {
            namespace.client_id = Set(client_id);
        }
        if let Some(client_secret) = update_namespace_object.client_secret {
            namespace.client_secret = Set(client_secret);
        }

        namespace.updated_at = Set(Utc::now());

        let updated_namespace = namespace.update(&*self.db).await.map_err(|err| ServerError::DBError(err))?;

        Ok(HttpResponse::Ok().json(updated_namespace))
    }
    
    pub async fn delete_namespace(&self, user_id: Uuid, namespace_id: Uuid) -> Result<HttpResponse, ServerError> {
        let found_namespace_junc = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::UserId.eq(user_id))
            .filter(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .one(&*self.db)
            .await
            .map_err(ServerError::DBError)?;

        let namespace_junc = match found_namespace_junc {
            Some(junction) => junction,
            None => return Err(ServerError::PermissionDenied),
        };
        
        let namespace_junc_active_model = namespace_junc.into_active_model();

        let delete_namespace_result = NamespaceEntity::delete_by_id(namespace_id)
            .exec(&*self.db)
            .await
            .map_err(ServerError::DBError)?;

        if delete_namespace_result.rows_affected == 0 {
            return Err(ServerError::NamespaceNotFound);
        }

        // Delete junction too
        let delete_junction_result = UserNamespaceJunctionEntity::delete(namespace_junc_active_model)
            .exec(&*self.db)
            .await
            .map_err(ServerError::DBError)?;

        if delete_junction_result.rows_affected == 0 {
            // Log this as it's unusual if we've just deleted the namespace but not the junction
            return Err(ServerError::NamespaceNotFound);
        }

        Ok(HttpResponse::Ok().finish())
    }
}
