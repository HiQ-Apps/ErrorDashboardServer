use sea_orm::QueryOrder;
use sea_orm::{
    entity::prelude::*, ActiveValue, DatabaseConnection, EntityTrait, IntoActiveModel, QuerySelect,
    TransactionTrait,
};
use shared_types::extra_dtos::QueryParams;
use shared_types::notification_dtos::{
    CreateNotificationDTO, GetNotificationResponse, NotificationDTO,
};
use std::sync::Arc;

use crate::config::Config;
use crate::models::notification_model::{
    ActiveModel, Entity as NotificationEntity, Model as NotificationModel,
};
use crate::shared::utils::errors::{ExternalError, ServerError};

pub struct NotificationService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl NotificationService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_notification(
        &self,
        create_notification: CreateNotificationDTO,
    ) -> Result<(), ServerError> {
        let db = &*self.db;
        // Create notification
        let user_id = create_notification.user_id.clone();
        let notification_model: NotificationModel = create_notification.into();
        let active_notification = notification_model.into_active_model();

        let _ = NotificationEntity::insert(active_notification)
            .exec(db)
            .await
            .map_err(|err| {
                ServerError::ExternalError(ExternalError::DB(err));
            });

        // Delete the oldest notification if the user has more than 10 notifications
        let extra_notifications = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .order_by_desc(<NotificationEntity as EntityTrait>::Column::CreatedAt)
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

    pub async fn get_notifications_by_user_id(
        &self,
        user_id: Uuid,
        query_params: QueryParams,
    ) -> Result<GetNotificationResponse, ServerError> {
        // Get notifications by user id
        let offset = query_params.offset;
        let limit = query_params.limit;

        let db = &*self.db;
        let notifications = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<NotificationEntity as EntityTrait>::Column::IsRead.eq(false))
            .order_by_desc(<NotificationEntity as EntityTrait>::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let notifications = notifications
            .iter()
            .map(|notification| NotificationDTO {
                id: notification.id.clone(),
                user_id: notification.user_id.clone(),
                source: notification.source.clone(),
                title: notification.title.clone(),
                text: notification.text.clone(),
                is_read: notification.is_read.clone(),
                created_at: notification.created_at.clone(),
            })
            .collect();

        let unread_count = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<NotificationEntity as EntityTrait>::Column::IsRead.eq(false))
            .count(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(GetNotificationResponse {
            notifications,
            unread_count,
        })
    }

    pub async fn seen_notification(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<(), ServerError> {
        // Mark notification as seen
        let db = &*self.db;
        let notification = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<NotificationEntity as EntityTrait>::Column::Id.eq(notification_id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let notification = notification.unwrap();
        let mut active_notification = notification.into_active_model();
        active_notification.is_read = ActiveValue::Set(true);
        active_notification
            .update(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(())
    }

    pub async fn batch_seen_notifications(
        &self,
        user_id: Uuid,
        notification_ids: Vec<Uuid>,
    ) -> Result<(), ServerError> {
        let db = &*self.db;
        // Mark notifications as seen
        let notifications = NotificationEntity::find()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .filter(<NotificationEntity as EntityTrait>::Column::Id.is_in(notification_ids))
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        for notification in notifications {
            let mut active_notification = notification.into_active_model();
            active_notification.is_read = ActiveValue::Set(true);
            active_notification
                .update(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        }

        Ok(())
    }

    pub async fn clear_notifications(&self, user_id: Uuid) -> Result<(), ServerError> {
        // Clear notifications
        let db = &*self.db;
        <NotificationEntity as EntityTrait>::delete_many()
            .filter(<NotificationEntity as EntityTrait>::Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(())
    }

    pub async fn delete_notification(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<(), ServerError> {
        // Delete notification
        Ok(())
    }
}
