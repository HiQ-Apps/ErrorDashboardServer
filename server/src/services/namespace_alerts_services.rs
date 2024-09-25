use uuid::Uuid;
use sea_orm::{DatabaseConnection, IntoActiveModel, EntityTrait, Condition, QueryOrder, QuerySelect, QueryFilter, TransactionTrait, ColumnTrait, ModelTrait};
use std::sync::Arc;

use shared_types::namespace_alert_dtos::{CreateNamespaceAlertRequestDTO, NamespaceAlertSubscriptionRequestDTO, ShortNamespaceAlertDTO, UpdateNamespaceAlertRequestDTO};
use crate::config::Config;
use crate::shared::utils::errors::{ExternalError, ServerError, QueryError};
use crate::models::namespace_alerts_model::{Model as NamespaceAlertModel, Entity as NamespaceAlertEntity};
use crate::models::namespace_alert_user_junction_model::{Model as NamespaceAlertUserJunctionModel, Entity as NamespaceAlertUserJunctionEntity};

pub struct NamespaceAlertsService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl NamespaceAlertsService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_namespace_alert(&self, new_namespace_alert: CreateNamespaceAlertRequestDTO) -> Result<(), ServerError> {
        let db = &*self.db;
        let transaction = db.begin().await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;


        let uid = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        let namespace_alert = NamespaceAlertModel {
            id: uid,
            namespace_id: new_namespace_alert.namespace_id,
            alert_method: new_namespace_alert.alert_method,
            error_name: new_namespace_alert.error_name,
            path: new_namespace_alert.path,
            line: new_namespace_alert.line,
            message: new_namespace_alert.message,
            stack_trace: new_namespace_alert.stack_trace,
            count_threshold: new_namespace_alert.count_threshold,
            time_window: new_namespace_alert.time_window,
            unresolved_time_threshold: new_namespace_alert.unresolved_time_threshold,
            rate_threshold: new_namespace_alert.rate_threshold,
            rate_time_window: new_namespace_alert.rate_time_window,
            created_at: now,
            updated_at: now,
        }.into_active_model();
        
        if let Err(err) = NamespaceAlertEntity::insert(namespace_alert).exec(&transaction).await {
            transaction.rollback().await.map_err(ExternalError::from)?;
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }

        Ok(())
    }

    pub async fn delete_namespace_alert(&self, alert_id: Uuid) -> Result<(), ServerError> {
        let db = &*self.db;

        let namespace_alert = NamespaceAlertEntity::find()
            .filter(<NamespaceAlertEntity as EntityTrait>::Column::Id.eq(alert_id))
            .one(db)
            .await
            .map_err(ExternalError::from)?;

        if let Some(namespace_alert) = namespace_alert {
            namespace_alert.delete(db).await.map_err(ExternalError::from)?;
        }
        Ok(())
    }

    pub async fn get_namespace_alerts_by_namespace_id(&self, namespace_id: Uuid) -> Result<Vec<ShortNamespaceAlertDTO>, ServerError> {
        let db = &*self.db;

        let namespace_alerts = NamespaceAlertEntity::find()
            .filter(<NamespaceAlertEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .all(db)
            .await
            .map_err(ExternalError::from)?;

        let mut alerts = Vec::new();
        
        namespace_alerts.iter().for_each(|alert| {
            alerts.push(ShortNamespaceAlertDTO {
                id: alert.id,
                namespace_id: alert.namespace_id,
                alert_method: alert.alert_method.clone(),
                error_name: alert.error_name.clone(),
                path: alert.path.clone(),
                line: alert.line.clone(),
                message: alert.message.clone(),
                stack_trace: alert.stack_trace.clone(),
                count_threshold: alert.count_threshold,
                time_window: alert.time_window,
                unresolved_time_threshold: alert.unresolved_time_threshold,
                rate_threshold: alert.rate_threshold,
                rate_time_window: alert.rate_time_window,
            });
        });

        Ok(alerts)
    }
            
    
    pub async fn get_namespace_alerts_by_user_id(&self, user_id: Uuid) -> Result<Vec<ShortNamespaceAlertDTO>, ServerError> {
        let db = &*self.db;

        let user_alerts = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(user_id))
            .all(db)
            .await
            .map_err(ExternalError::from)?;

        let mut namespace_alert_ids = Vec::new();
        
        user_alerts.iter().for_each(|alert| {
            namespace_alert_ids.push(alert.namespace_alert_id);
        });

        let found_namespace_alerts = NamespaceAlertEntity::find()
            .filter(<NamespaceAlertEntity as EntityTrait>::Column::Id.eq(namespace_alert_ids))
            .all(db)
            .await
            .map_err(ExternalError::from)?;

        let mut alerts = Vec::new();
        
        found_namespace_alerts.iter().for_each(|alert| {
            alerts.push(ShortNamespaceAlertDTO {
                id: alert.id,
                namespace_id: alert.namespace_id,
                alert_method: alert.alert_method.clone(),
                error_name: alert.error_name.clone(),
                path: alert.path.clone(),
                line: alert.line.clone(),
                message: alert.message.clone(),
                stack_trace: alert.stack_trace.clone(),
                count_threshold: alert.count_threshold,
                time_window: alert.time_window,
                unresolved_time_threshold: alert.unresolved_time_threshold,
                rate_threshold: alert.rate_threshold,
                rate_time_window: alert.rate_time_window,
            });
        });

        Ok(alerts)
    }

    pub async fn subscribe_user_to_namespace_alert(&self, subscription_data: NamespaceAlertSubscriptionRequestDTO) -> Result<(), ServerError> {
        let check_user_in_namespace = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::NamespaceAlertId.eq(subscription_data.namespace_alert_id))
            .one(&*self.db)
            .await
            .map_err(ExternalError::from)?;

        if check_user_in_namespace.is_none() {
            return Err(ServerError::QueryError(QueryError::NotFound));
        }

        let namespace = NamespaceAlertUserJunctionModel {
            id: Uuid::new_v4(),
            namespace_alert_id: subscription_data.namespace_alert_id,
            user_id: subscription_data.user_id,
        }.into_active_model();

        if let Err(err) = NamespaceAlertUserJunctionEntity::insert(namespace).exec(&*self.db).await {
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }
        
        Ok(())
    }

    pub async fn unsubscribe_user_from_namespace_alert(&self, subscription_data: NamespaceAlertSubscriptionRequestDTO) -> Result<(), ServerError> {
        let check_user_in_namespace = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::NamespaceAlertId.eq(subscription_data.namespace_alert_id))
            .one(&*self.db)
            .await
            .map_err(ExternalError::from)?;

        if check_user_in_namespace.is_none() {
            return Err(ServerError::QueryError(QueryError::NotFound));
        }

        let namespace = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::NamespaceAlertId.eq(subscription_data.namespace_alert_id))
            .one(&*self.db)
            .await
            .map_err(ExternalError::from)?;

        if let Some(namespace) = namespace {
            namespace.delete(&*self.db).await.map_err(ExternalError::from)?;
        }

        Ok(())
    }

    pub async fn update_namespace_alert(&self, updated_namespace_alert: UpdateNamespaceAlertRequestDTO) -> Result<(), ServerError> {
        Ok(())
    }
}


