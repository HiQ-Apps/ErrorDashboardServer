use sea_orm::QueryOrder;
use sea_orm::{entity::prelude::*, ActiveValue, EntityTrait, IntoActiveModel, DatabaseConnection, QuerySelect, TransactionTrait};
use shared_types::notification_dtos::CreateNotificationDTO;
use std::sync::Arc;

use crate::config::Config;
use crate::shared::utils::errors::{ExternalError, ServerError};
use crate::models::notification_model::{Entity as NotificationEntity, Model as NotificationModel, ActiveModel};

pub struct NotificationService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl NotificationService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_notification(&self, create_notification: CreateNotificationDTO) -> Result<(), ServerError> {
        let db = &*self.db;
        // Create notification
        let user_id = create_notification.user_id.clone();
        let notification_model: NotificationModel = create_notification.into();
        let active_notification = notification_model.into_active_model();

        let _ = NotificationEntity::insert(active_notification)
            .exec(db).await.map_err(|err| { ServerError::ExternalError(ExternalError::DB(err));
        });

        // Delete the oldest notification if the user has more than 10 notifications
        let extra_notifications = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .order_by_asc(<NotificationEntity as EntityTrait>::Column::CreatedAt)
            .offset(10)
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        // Delete the extra notifications
        for notification in extra_notifications {
            notification
                .delete(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        }

        Ok(())
    }

    pub async fn get_notifications_by_user_id(&self, user_id: Uuid) -> Result<Vec<NotificationModel>, ServerError> {
        // Get notifications by user id
        let db = &*self.db;
        let notifications = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(notifications)
    }

    pub async fn seen_notification(&self, user_id: Uuid, notification_id: Uuid) -> Result<(), ServerError> {
        // Mark notification as seen
        let db = &*self.db;
        let notification = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<NotificationEntity as EntityTrait>::Column::Id.eq(notification_id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let mut notification = notification.unwrap();
        notification.is_read = true;
        let active_notification = notification.into_active_model();
        active_notification.update(db).await.map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(())
    }

    pub async fn delete_notification(&self, user_id: Uuid, notification_id: Uuid) -> Result<(), ServerError> {
        // Delete notification
        Ok(())
    }
}
