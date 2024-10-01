use uuid::Uuid;
use sea_orm::{DatabaseConnection, IntoActiveModel, ActiveModelTrait, EntityTrait, ActiveValue, Condition, QueryOrder, QuerySelect, QueryFilter, TransactionTrait, ColumnTrait, ModelTrait};
use std::sync::Arc;

use shared_types::namespace_alert_dtos::{CreateNamespaceAlertRequestDTO, NamespaceAlertSubscriptionRequestDTO, ShortNamespaceAlertDTO, UpdateNamespaceAlertRequestDTO};
use crate::config::Config;
use crate::shared::utils::errors::{ExternalError, ServerError, QueryError};
use crate::models::namespace_alerts_model::{Model as NamespaceAlertModel, Entity as NamespaceAlertEntity};
use crate::models::namespace_alert_user_junction_model::{Model as NamespaceAlertUserJunctionModel, Entity as NamespaceAlertUserJunctionEntity};
use crate::models::user_namespace_junction_model::Entity as UserNamespaceJunctionEntity;

pub struct NamespaceAlertsService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl NamespaceAlertsService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_namespace_alert(&self, new_namespace_alert: CreateNamespaceAlertRequestDTO) -> Result<Uuid, ServerError> {
        let db = &*self.db;

        let uid = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        let namespace_alert = NamespaceAlertModel {
            id: uid,
            namespace_id: new_namespace_alert.namespace_id,
            alert_method: new_namespace_alert.alert_method,
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
        
        if let Err(err) = NamespaceAlertEntity::insert(namespace_alert).exec(db).await {
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }

        Ok(uid)
    }

    pub async fn delete_namespace_alert(&self, alert_id: Uuid) -> Result<(), ServerError> {
        let db = &*self.db;

        let namespace_alert = NamespaceAlertEntity::find()
            .filter(<NamespaceAlertEntity as EntityTrait>::Column::Id.eq(alert_id))
            .one(db)
            .await
            .map_err(ExternalError::from)?;

        if namespace_alert.is_none() {
            return Err(ServerError::QueryError(QueryError::NamespaceAlertNotFound));
        }

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
        let check_user_in_namespace = UserNamespaceJunctionEntity::find()
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<UserNamespaceJunctionEntity as EntityTrait>::Column::NamespaceId.eq(subscription_data.namespace_id))
            .one(&*self.db)
            .await
            .map_err(ExternalError::from)?;

        if check_user_in_namespace.is_none() {
            return Err(ServerError::QueryError(QueryError::UserNotNamespaceMember));
        }

        let check_user_subscription = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::NamespaceAlertId.eq(subscription_data.namespace_alert_id))
            .one(&*self.db)
            .await
            .map_err(ExternalError::from)?;

        if check_user_subscription.is_some() {
            return Err(ServerError::QueryError(QueryError::UserAlreadySubscribed));
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
        let db = &*self.db;
        
        let check_user_in_namespace = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::NamespaceAlertId.eq(subscription_data.namespace_alert_id))
            .one(db)
            .await
            .map_err(ExternalError::from)?;

        if check_user_in_namespace.is_none() {
            return Err(ServerError::QueryError(QueryError::UserNotFound));
        }

        let namespace = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::UserId.eq(subscription_data.user_id))
            .filter(<NamespaceAlertUserJunctionEntity as EntityTrait>::Column::NamespaceAlertId.eq(subscription_data.namespace_alert_id))
            .one(db)
            .await
            .map_err(ExternalError::from)?;

        if let Some(namespace) = namespace {
            namespace.delete(db).await.map_err(ExternalError::from)?;
        }
        Ok(())
    }

    pub async fn update_namespace_alert(&self, alert_id: Uuid, updated_namespace_alert: UpdateNamespaceAlertRequestDTO) -> Result<(), ServerError> {
        let db = &*self.db;
        let now = chrono::Utc::now();

        let namespace_alert = NamespaceAlertEntity::find()
            .filter(<NamespaceAlertEntity as EntityTrait>::Column::Id.eq(alert_id))
            .one(db)
            .await;

        let mut updated_alert = match namespace_alert {
            Ok(Some(alert)) => alert.into_active_model(),
            Ok(None) => return Err(ServerError::QueryError(QueryError::NamespaceAlertNotFound)),
            Err(err) => return Err(ServerError::ExternalError(ExternalError::DB(err))),
        };

        if let Some(alert_method) = updated_namespace_alert.alert_method {
            updated_alert.alert_method = ActiveValue::Set(alert_method);
        }

        if let Some(path) = updated_namespace_alert.path {
            updated_alert.path = ActiveValue::Set(Some(path));
        }

        if let Some(line) = updated_namespace_alert.line {
            updated_alert.line = ActiveValue::Set(Some(line));
        }

        if let Some(message) = updated_namespace_alert.message {
            updated_alert.message = ActiveValue::Set(Some(message));
        }

        if let Some(stack_trace) = updated_namespace_alert.stack_trace {
            updated_alert.stack_trace = ActiveValue::Set(Some(stack_trace));
        }

        if let Some(count_threshold) = updated_namespace_alert.count_threshold {
            updated_alert.count_threshold = ActiveValue::Set(Some(count_threshold));
        }

        if let Some(time_window) = updated_namespace_alert.time_window {
            updated_alert.time_window = ActiveValue::Set(Some(time_window));
        }

        if let Some(unresolved_time_threshold) = updated_namespace_alert.unresolved_time_threshold {
            updated_alert.unresolved_time_threshold = ActiveValue::Set(Some(unresolved_time_threshold));
        }

        if let Some(rate_threshold) = updated_namespace_alert.rate_threshold {
            updated_alert.rate_threshold = ActiveValue::Set(Some(rate_threshold));
        }

        if let Some(rate_time_window) = updated_namespace_alert.rate_time_window {
            updated_alert.rate_time_window = ActiveValue::Set(Some(rate_time_window));
        }

        updated_alert.updated_at = ActiveValue::Set(now);

        if let Err(err) = updated_alert.update(db).await {
            return Err(ServerError::ExternalError(ExternalError::DB(err)));
        }

        Ok(())
    }
}


