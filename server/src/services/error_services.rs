use actix_web::Result;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sea_orm::sea_query::Query;
use sea_orm::ActiveValue::NotSet;
use sea_orm::Set;
use sea_orm::{
    entity::prelude::*, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, JoinType,
    QueryOrder, QuerySelect,
};
use shared_types::notification_dtos::NotificationDTO;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::managers::notification_manager::NotificationServer;
use crate::models::error_model::{Entity as ErrorEntity, Model as ErrorModel};
use crate::models::error_tag_model::{
    ActiveModel as ActiveTagModel, Entity as TagEntity, Model as TagModel,
};
use crate::models::namespace_alert_user_junction_model::Entity as NamespaceAlertUserJunctionEntity;
use crate::models::namespace_alerts_model::{Entity as NamespaceAlertEntity, ActiveModel as NamespaceAlertActiveModel};
use crate::models::namespace_model::Entity as NamespaceEntity;
use crate::models::notification_model::{Entity as NotificationEntity, Model as NotificationModel};
use crate::models::user_model::Entity as UserEntity;
use crate::shared::utils::discord::DiscordHandler;
use crate::shared::utils::errors::{ExternalError, QueryError, RequestError, ServerError};
use crate::shared::utils::mailing::{send_email, EmailContent};
use crate::shared::utils::parse::{parse_stack_trace, StackTraceInfo};
use shared_types::error_dtos::{
    AggregateErrorDTO, CreateErrorDTO, CreateErrorRequest, ErrorDTO, ErrorMetaDTO,
    GroupedAggregateErrorCountDTO, UpdateErrorDTO,
};
use shared_types::tag_dtos::{CreateTagRequestDTO, ShortTagDTO, TagDTO};

pub struct ErrorService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl ErrorService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_error(
        &self,
        discord_handler: &DiscordHandler,
        error: CreateErrorRequest,
        namespace_client_id: Uuid,
        notification_manager: &Arc<NotificationServer>,
    ) -> Result<CreateErrorDTO, ServerError> {
        let now = Utc::now();
        let configs = &*self.configs;
        let db = &*self.db;

        let mut stack_trace_info: StackTraceInfo = StackTraceInfo::default();
        let error_stack_trace = error.stack_trace.clone();

        match parse_stack_trace(&error_stack_trace) {
            Ok(info) => stack_trace_info = info,
            Err(_) => {
                return Err(ServerError::RequestError(
                    RequestError::StackTraceParsingError,
                ))
            }
        }

        // Find the namespace for this error
        let found_namespace = NamespaceEntity::find()
            .filter(
                <NamespaceEntity as sea_orm::EntityTrait>::Column::ClientId.eq(namespace_client_id),
            )
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?
            .ok_or(ServerError::QueryError(QueryError::NamespaceNotFound))?;

        // Find alerts for this namespace
        let found_alerts = NamespaceAlertEntity::find()
            .filter(
                <NamespaceAlertEntity as sea_orm::EntityTrait>::Column::NamespaceId
                    .eq(found_namespace.id),
            )
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let mut alerts_sent: Vec<Uuid> = Vec::new();
        
        // Find subscribed users for each alert
        for alert in found_alerts {
            // // we need to filter the error based on one of these fields
            if alert.triggered == true {
                continue;
            } 

            if let Some(alert_path) = alert.path.clone() {
                if alert_path != stack_trace_info.file_path {
                    continue;
                }
            }
            if let Some(alert_line) = alert.line.clone() {
                if alert_line != stack_trace_info.line_number {
                    continue;
                }
            }
            if let Some(alert_message) = alert.message.clone() {
                if alert_message != error.message {
                    continue;
                }
            }

            let subscribed_users = NamespaceAlertUserJunctionEntity::find()
            .filter(<NamespaceAlertUserJunctionEntity as sea_orm::EntityTrait>::Column::NamespaceAlertId.eq(alert.id))
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

            for user_alert_junction in subscribed_users {
                // Check type of alert (discord,email, etc.) and send + notify users.
                // TODO: Add ability to disable notifications for users
                match alert.alert_method.as_str() {
                    "email" => {
                        if let Some(count_threshold) = alert.count_threshold {
                            let time_window = alert.time_window.unwrap();
                            let time_window_minutes = time_window / 60000;
                            let time_window_start = now - Duration::minutes(time_window_minutes);

                            // Find all errors in the time window
                            let mut query = ErrorEntity::find()
                                .filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId
                                        .eq(found_namespace.id),
                                )
                                .filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt
                                        .gt(time_window_start),
                                );
                            // Add filter for whichever alert field we are looking for

                            if let Some(alert_path) = &alert.path {
                                query = query.filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::Path
                                        .eq(alert_path.clone()),
                                );
                            }
                            if let Some(alert_line) = &alert.line {
                                query = query.filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::Line
                                        .eq(alert_line.clone()),
                                );
                            }
                            if let Some(alert_message) = &alert.message {
                                query = query.filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::Message
                                        .eq(alert_message.clone()),
                                );
                            }

                            let error_count = query.count(&*self.db).await.map_err(|err| {
                                ServerError::ExternalError(ExternalError::DB(err))
                            })?;

                            // Check if error count is equal to threshold because we don't want to spam users after it hits the threshold
                            if error_count > count_threshold as u64 {
                                let content = EmailContent {
                      greeting: "Alert Notice!".to_string(), 
                      main_message: format!("An error alert has been triggered for a namespace you are subscribed to by the ID of {}", alert.namespace_id),
                      body: format!("Please log in to your account to view the error details and resolve the issue. {}", configs.domain),
                      dynamic_content: Some(format!("Alert Details:\nError Count Triggered: {}.\nTime Triggered: {}", count_threshold, now)),
                    };

                                // Get user email for each user and send email
                                let find_user = UserEntity::find()
                                    .filter(
                                        <UserEntity as sea_orm::EntityTrait>::Column::Id
                                            .eq(user_alert_junction.user_id),
                                    )
                                    .one(db)
                                    .await
                                    .map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?
                                    .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

                                let create_notification = NotificationDTO {
                        id: Uuid::new_v4(),
                        user_id: find_user.id,
                        title: "Alert Notification".to_string(),
                        text: format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {}", alert.id, alert.namespace_id),
                        source: "HiGuard Alert System".to_string(),
                        is_read: false,
                        created_at: now,
                    };
                                let broadcast_notification = create_notification.clone();

                                let notification_model =
                                    NotificationModel::from(create_notification);
                                let active_create_notification =
                                    notification_model.into_active_model();

                                NotificationEntity::insert(active_create_notification)
                                    .exec(db)
                                    .await
                                    .map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?;

                                notification_manager
                                    .broadcast_notification(broadcast_notification, &find_user.id)
                                    .await;

                                send_email(configs, &find_user.email, "Error Alert", &content)
                                    .map_err(|err| ServerError::from(err))?;

                                    alerts_sent.push(alert.id);

                            } else if let Some(unresolved_time_threshold) =
                                alert.unresolved_time_threshold
                            {
                                let time_window = unresolved_time_threshold;
                                let time_window_minutes = time_window / 60000;
                                let time_window_start =
                                    now - Duration::minutes(time_window_minutes);
                                // Find all errors that are unresolved in the unresolved time threshold
                                let errors = ErrorEntity::find()
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId
                                            .eq(found_namespace.id),
                                    )
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Resolved
                                            .eq(false),
                                    )
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt
                                            .gt(time_window_start),
                                    )
                                    .all(&*self.db)
                                    .await
                                    .map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?;

                                let error_ids =
                                    errors.iter().map(|error| error.id).collect::<Vec<Uuid>>();

                                // Warn users if the error has been unresolved for the threshold time
                                if errors.len() > 0 {
                                    let content = EmailContent {
                          greeting: "Alert Notice!".to_string(), 
                          main_message: format!("An error alert has been triggered for a namespace you are subscribed to by the ID of {}", alert.namespace_id),
                          body: format!("Please log in to your account to view the error details and resolve the issue. {}", configs.domain),
                          dynamic_content: Some(format!("Alert Details:\nErrors unresolved within time: {} Errors. \nError IDs: {:?}", errors.len(), error_ids)),
                          };

                                    let find_user = UserEntity::find()
                                        .filter(
                                            <UserEntity as sea_orm::EntityTrait>::Column::Id
                                                .eq(user_alert_junction.user_id),
                                        )
                                        .one(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?
                                        .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

                                    let create_notification = NotificationDTO {
                              id: Uuid::new_v4(),
                              user_id: find_user.id,
                              title: "Alert Notification".to_string(),
                              text: format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {}", alert.id, alert.namespace_id),
                              source: "HiGuard Alert System".to_string(),
                              is_read: false,
                              created_at: now,
                          };
                                    let broadcast_notification = create_notification.clone();

                                    let notification_model =
                                        NotificationModel::from(create_notification);
                                    let active_create_notification =
                                        notification_model.into_active_model();

                                    NotificationEntity::insert(active_create_notification)
                                        .exec(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?;

                                    notification_manager
                                        .broadcast_notification(
                                            broadcast_notification,
                                            &find_user.id,
                                        )
                                        .await;

                                    send_email(configs, &find_user.email, "Error Alert", &content)
                                        .map_err(|err| ServerError::from(err))?;

                                    alerts_sent.push(alert.id);
                                }
                            } else if let Some(rate_threshold) = alert.rate_threshold {
                                let time_window = alert.rate_time_window.unwrap();
                                let time_window_minutes = time_window / 60000;
                                let time_window_start: DateTime<Utc> =
                                    now - Duration::minutes(time_window_minutes);

                                // Find all errors in the time window
                                let mut error_query = ErrorEntity::find()
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId
                                            .eq(found_namespace.id),
                                    )
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt
                                            .gt(time_window_start),
                                    );

                                if let Some(alert_path) = &alert.path {
                                    error_query = error_query.filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Path
                                            .eq(alert_path.clone()),
                                    );
                                }
                                if let Some(alert_line) = &alert.line {
                                    error_query = error_query.filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Line
                                            .eq(alert_line.clone()),
                                    );
                                }
                                if let Some(alert_message) = &alert.message {
                                    error_query = error_query.filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Message
                                            .eq(alert_message.clone()),
                                    );
                                }

                                let error_count =
                                    error_query.count(&*self.db).await.map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?;

                                // Check if the rate at which errors are being created is greater than the threshold
                                // Calculate rate by dividing the number of errors by the time window
                                // Time is all calculated in milliseconds so we need to convert the time window to milliseconds
                                let rate = error_count as f64 / (time_window as f64 * 60.0);

                                if rate > rate_threshold as f64 {
                                    let content = EmailContent {
                          greeting: "Alert Notice!".to_string(), 
                          main_message: format!("An error alert has been triggered for a namespace you are subscribed to by the ID of {}", alert.namespace_id),
                          body: format!("Please log in to your account to view the error details and resolve the issue. {}", configs.domain),
                          dynamic_content: Some(format!("Alert Details:\nRate: {} errors per minute. \nThreshold: {} errors per minute. \nAlert ID: {}", rate, alert.rate_threshold.unwrap(), alert.id)),
                          };

                                    let find_user = UserEntity::find()
                                        .filter(
                                            <UserEntity as sea_orm::EntityTrait>::Column::Id
                                                .eq(user_alert_junction.user_id),
                                        )
                                        .one(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?
                                        .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

                                    let create_notification = NotificationDTO {
                              id: Uuid::new_v4(),
                              user_id: find_user.id,
                              title: "Alert Notification".to_string(),
                              text: format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {}", alert.id, alert.namespace_id),
                              source: "HiGuard Alert System".to_string(),
                              is_read: false,
                              created_at: now,
                          };
                                    let broadcast_notification = create_notification.clone();

                                    let notification_model =
                                        NotificationModel::from(create_notification);
                                    let active_create_notification =
                                        notification_model.into_active_model();

                                    NotificationEntity::insert(active_create_notification)
                                        .exec(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?;

                                    notification_manager
                                        .broadcast_notification(
                                            broadcast_notification,
                                            &find_user.id,
                                        )
                                        .await;

                                    send_email(configs, &find_user.email, "Error Alert", &content)
                                        .map_err(|err| ServerError::from(err))?;

                                    alerts_sent.push(alert.id);
                                }
                            } else {
                                continue;
                            }
                        }
                    }
                    "discord" => {
                        if let Some(count_threshold) = alert.count_threshold {
                            let time_window = alert.time_window.unwrap();
                            let time_window_minutes = time_window / 60000;
                            let time_window_start = now - Duration::minutes(time_window_minutes);

                            // Find all errors in the time window
                            let mut query = ErrorEntity::find()
                                .filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId
                                        .eq(found_namespace.id),
                                )
                                .filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt
                                        .gt(time_window_start),
                                );
                            // Add filter for whichever alert field we are looking for

                            if let Some(alert_path) = &alert.path {
                                query = query.filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::Path
                                        .eq(alert_path.clone()),
                                );
                            }
                            if let Some(alert_line) = &alert.line {
                                query = query.filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::Line
                                        .eq(alert_line.clone()),
                                );
                            }
                            if let Some(alert_message) = &alert.message {
                                query = query.filter(
                                    <ErrorEntity as sea_orm::EntityTrait>::Column::Message
                                        .eq(alert_message.clone()),
                                );
                            }

                            let error_count = query.count(&*self.db).await.map_err(|err| {
                                ServerError::ExternalError(ExternalError::DB(err))
                            })?;

                            // Check if error count is equal to threshold because we don't want to spam users after it hits the threshold
                            if error_count > count_threshold as u64 {
                                // Get user email for each user and send email
                                let find_user = UserEntity::find()
                                    .filter(
                                        <UserEntity as sea_orm::EntityTrait>::Column::Id
                                            .eq(user_alert_junction.user_id),
                                    )
                                    .one(db)
                                    .await
                                    .map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?
                                    .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

                                let create_notification = NotificationDTO {
                        id: Uuid::new_v4(),
                        user_id: find_user.id,
                        title: "Alert Notification".to_string(),
                        text: format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {}", alert.id, alert.namespace_id),
                        source: "HiGuard Alert System".to_string(),
                        is_read: false,
                        created_at: now,
                    };
                                let broadcast_notification = create_notification.clone();

                                let notification_model =
                                    NotificationModel::from(create_notification);
                                let active_create_notification =
                                    notification_model.into_active_model();

                                NotificationEntity::insert(active_create_notification)
                                    .exec(db)
                                    .await
                                    .map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?;

                                notification_manager
                                    .broadcast_notification(broadcast_notification, &find_user.id)
                                    .await;

                                let content = format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {} at {}.", alert.id, alert.namespace_id, alert.created_at);
                                let channel_id = alert.discord_channel_id.as_ref().ok_or(
                                    ServerError::QueryError(QueryError::DiscordChannelNotFound),
                                )?;

                                let channel_id: u64 = channel_id.parse().map_err(|_| {
                                    ServerError::QueryError(QueryError::DiscordChannelNotFound)
                                })?;

                                discord_handler
                                    .send_discord_alert(channel_id, &content)
                                    .await
                                    .map_err(|err| ServerError::from(err))?;

                                alerts_sent.push(alert.id);

                            } else if let Some(unresolved_time_threshold) =
                                alert.unresolved_time_threshold
                            {
                                let time_window = unresolved_time_threshold;
                                let time_window_minutes = time_window / 60000;
                                let time_window_start =
                                    now - Duration::minutes(time_window_minutes);
                                // Find all errors that are unresolved in the unresolved time threshold
                                let errors = ErrorEntity::find()
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId
                                            .eq(found_namespace.id),
                                    )
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Resolved
                                            .eq(false),
                                    )
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt
                                            .gt(time_window_start),
                                    )
                                    .all(&*self.db)
                                    .await
                                    .map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?;

                                let error_ids =
                                    errors.iter().map(|error| error.id).collect::<Vec<Uuid>>();

                                // Warn users if the error has been unresolved for the threshold time
                                if errors.len() > 0 {
                                    let find_user = UserEntity::find()
                                        .filter(
                                            <UserEntity as sea_orm::EntityTrait>::Column::Id
                                                .eq(user_alert_junction.user_id),
                                        )
                                        .one(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?
                                        .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

                                    let create_notification = NotificationDTO {
                              id: Uuid::new_v4(),
                              user_id: find_user.id,
                              title: "Alert Notification".to_string(),
                              text: format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {}", alert.id, alert.namespace_id),
                              source: "HiGuard Alert System".to_string(),
                              is_read: false,
                              created_at: now,
                          };
                                    let broadcast_notification = create_notification.clone();

                                    let notification_model =
                                        NotificationModel::from(create_notification);
                                    let active_create_notification =
                                        notification_model.into_active_model();

                                    NotificationEntity::insert(active_create_notification)
                                        .exec(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?;

                                    notification_manager
                                        .broadcast_notification(
                                            broadcast_notification,
                                            &find_user.id,
                                        )
                                        .await;

                                    let content = format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {} at {}.", alert.id, alert.namespace_id, alert.created_at);
                                    let channel_id = alert.discord_channel_id.as_ref().ok_or(
                                        ServerError::QueryError(QueryError::DiscordChannelNotFound),
                                    )?;

                                    let channel_id: u64 = channel_id.parse().map_err(|_| {
                                        ServerError::QueryError(QueryError::DiscordChannelNotFound)
                                    })?;

                                    discord_handler
                                        .send_discord_alert(channel_id, &content)
                                        .await
                                        .map_err(|err| ServerError::from(err))?;

                                    alerts_sent.push(alert.id);
                                }
                            } else if let Some(rate_threshold) = alert.rate_threshold {
                                let time_window = alert.rate_time_window.unwrap();
                                let time_window_minutes = time_window / 60000;
                                let time_window_start: DateTime<Utc> =
                                    now - Duration::minutes(time_window_minutes);

                                // Find all errors in the time window
                                let mut error_query = ErrorEntity::find()
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::NamespaceId
                                            .eq(found_namespace.id),
                                    )
                                    .filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::CreatedAt
                                            .gt(time_window_start),
                                    );

                                if let Some(alert_path) = &alert.path {
                                    error_query = error_query.filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Path
                                            .eq(alert_path.clone()),
                                    );
                                }
                                if let Some(alert_line) = &alert.line {
                                    error_query = error_query.filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Line
                                            .eq(alert_line.clone()),
                                    );
                                }
                                if let Some(alert_message) = &alert.message {
                                    error_query = error_query.filter(
                                        <ErrorEntity as sea_orm::EntityTrait>::Column::Message
                                            .eq(alert_message.clone()),
                                    );
                                }

                                let error_count =
                                    error_query.count(&*self.db).await.map_err(|err| {
                                        ServerError::ExternalError(ExternalError::DB(err))
                                    })?;

                                // Check if the rate at which errors are being created is greater than the threshold
                                // Calculate rate by dividing the number of errors by the time window
                                // Time is all calculated in milliseconds so we need to convert the time window to milliseconds
                                let rate = error_count as f64 / (time_window as f64 * 60.0);

                                if rate > rate_threshold as f64 {
                                    let find_user = UserEntity::find()
                                        .filter(
                                            <UserEntity as sea_orm::EntityTrait>::Column::Id
                                                .eq(user_alert_junction.user_id),
                                        )
                                        .one(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?
                                        .ok_or(ServerError::QueryError(QueryError::UserNotFound))?;

                                    let create_notification = NotificationDTO {
                              id: Uuid::new_v4(),
                              user_id: find_user.id,
                              title: "Alert Notification".to_string(),
                              text: format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {}", alert.id, alert.namespace_id),
                              source: "HiGuard Alert System".to_string(),
                              is_read: false,
                              created_at: now,
                          };
                                    let broadcast_notification = create_notification.clone();

                                    let notification_model =
                                        NotificationModel::from(create_notification);
                                    let active_create_notification =
                                        notification_model.into_active_model();

                                    NotificationEntity::insert(active_create_notification)
                                        .exec(&*self.db)
                                        .await
                                        .map_err(|err| {
                                            ServerError::ExternalError(ExternalError::DB(err))
                                        })?;

                                    notification_manager
                                        .broadcast_notification(
                                            broadcast_notification,
                                            &find_user.id,
                                        )
                                        .await;
                                    let content = format!("Alert {} has been triggered for a namespace you are subscribed to by the ID of {} at {}.", alert.id, alert.namespace_id, alert.created_at);
                                    let channel_id = alert.discord_channel_id.as_ref().ok_or(
                                        ServerError::QueryError(QueryError::DiscordChannelNotFound),
                                    )?;

                                    let channel_id: u64 = channel_id.parse().map_err(|_| {
                                        ServerError::QueryError(QueryError::DiscordChannelNotFound)
                                    })?;

                                    discord_handler
                                        .send_discord_alert(channel_id, &content)
                                        .await
                                        .map_err(|err| ServerError::from(err))?;

                                    alerts_sent.push(alert.id);
                                }
                            } else {
                                continue;
                            }

                        
                        }
                    }
                    _ => {
                        return Err(ServerError::QueryError(QueryError::AlertTypeNotFound));
                    }
                }
            }
        }

        // Batch update alerts triggered
        let update_model = NamespaceAlertActiveModel {
            triggered: Set(true),
            id: NotSet,
            namespace_id: NotSet,
            alert_method: NotSet,
            discord_channel_id: NotSet,
            path: NotSet,
            line: NotSet,
            message: NotSet,
            stack_trace: NotSet,
            count_threshold: NotSet,
            time_window: NotSet,
            unresolved_time_threshold: NotSet,
            rate_threshold: NotSet,
            rate_time_window: NotSet,
            created_at: NotSet,
            updated_at: NotSet,
        };

        // Update alerts triggered
        NamespaceAlertEntity::update_many()
            .set(update_model)
            .filter(<NamespaceAlertEntity as sea_orm::EntityTrait>::Column::Id.is_in(alerts_sent))
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

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
            .exec(db)
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
                    .exec(db)
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

    pub async fn get_error_by_id(&self, id: Uuid) -> Result<ErrorDTO, ServerError> {
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

        let tags = Some(
            found_tags
                .into_iter()
                .map(|tag| ShortTagDTO {
                    id: tag.id,
                    tag_key: tag.tag_key,
                    tag_value: tag.tag_value,
                    tag_color: tag.tag_color,
                })
                .collect(),
        );

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

    pub async fn update_error(&self, error: UpdateErrorDTO) -> Result<UpdateErrorDTO, ServerError> {
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
                }
                .into_active_model();

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
            tags: Some(tag_list),
        })
    }

    pub async fn get_aggregate_errors_by_date(
        &self,
        namespace_id: Uuid,
        selected_date: NaiveDate,
        time_interval: i64,
        timezone: String,
    ) -> Result<Vec<AggregateErrorDTO>, ServerError> {
        let user_tz: Tz = timezone
            .parse()
            .map_err(|_| ServerError::QueryError(QueryError::InvalidTimestamp))?;

        let start_time_naive = selected_date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ServerError::QueryError(QueryError::InvalidTimestamp))?;
        let start_time_local = user_tz
            .from_local_datetime(&start_time_naive)
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
            let time_bucket_seconds =
                (error.created_at.timestamp() / (time_interval * 60)) * (time_interval * 60);
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
                    Condition::any().add(
                        <ErrorEntity as EntityTrait>::Column::Id.in_subquery(
                            Query::select()
                                .column(<TagEntity as EntityTrait>::Column::ErrorId)
                                .from(TagEntity)
                                .and_where(<TagEntity as EntityTrait>::Column::TagKey.eq(tag_key))
                                .and_where(
                                    <TagEntity as EntityTrait>::Column::TagValue.eq(tag_value),
                                )
                                .to_owned(),
                        ),
                    ),
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

        let error_meta = errors
            .into_iter()
            .map(|error| {
                ErrorMetaDTO {
                    id: error.id,
                    created_at: error.created_at,
                    // user_agent: error.user_agent,
                    resolved: error.resolved,
                }
            })
            .collect();

        Ok(error_meta)
    }

    pub async fn get_error_metrics_pie_chart(
        &self,
        namespace_id: Uuid,
        group_by: String,
    ) -> Result<Vec<GroupedAggregateErrorCountDTO>, ServerError> {
        let db: &DatabaseConnection = &*self.db;

        let aggregated_errors = if group_by == "tags" {
            let tags = TagEntity::find()
                .filter(
                    <TagEntity as EntityTrait>::Column::ErrorId.in_subquery(
                        Query::select()
                            .column(<ErrorEntity as EntityTrait>::Column::Id)
                            .from(ErrorEntity)
                            .and_where(
                                <ErrorEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id),
                            )
                            .to_owned(),
                    ),
                )
                .all(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

            let mut tag_map: HashMap<String, i64> = HashMap::new();

            for tag in tags {
                let tag_key = tag.tag_key.clone();
                let tag_value = tag.tag_value.clone();
                let tag_key_value = format!("{}:{}", tag_key, tag_value);

                *tag_map.entry(tag_key_value).or_insert(0) += 1;
            }

            tag_map
                .into_iter()
                .map(|(group_key, count)| GroupedAggregateErrorCountDTO { group_key, count })
                .collect()
        } else {
            let errors = <ErrorEntity as EntityTrait>::find()
                .filter(<ErrorEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
                .all(db)
                .await
                .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

            let mut error_map: HashMap<String, i64> = HashMap::new();

            for error in errors {
                let group_key = match group_by.as_str() {
                    "message" => error.message.clone(),
                    _ => error.message.clone(),
                };
                *error_map.entry(group_key).or_insert(0) += 1;
            }

            error_map
                .into_iter()
                .map(|(group_key, count)| GroupedAggregateErrorCountDTO { group_key, count })
                .collect()
        };

        Ok(aggregated_errors)
    }

    pub async fn get_unique_error_meta_by_namespace(
        &self,
        namespace_id: Uuid,
        filter: String,
    ) -> Result<Vec<String>, ServerError> {
        let db: &DatabaseConnection = &*self.db;

        let errors = ErrorEntity::find()
            .filter(<ErrorEntity as EntityTrait>::Column::NamespaceId.eq(namespace_id))
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        let mut unique_meta: Vec<String> = Vec::new();

        for error in errors {
            let meta = match filter.as_str() {
                "message" => error.message,
                "path" => error.path,
                "line" => error.line.to_string(),
                "stackTrace" => error.stack_trace,
                _ => error.message,
            };
            if !unique_meta.contains(&meta) {
                unique_meta.push(meta);
            }
        }
        Ok(unique_meta)
    }

}
