use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use shared_types::notification_dtos::{CreateNotificationDTO, NotificationDTO};
use uuid::Uuid;

use crate::models::user_model::Entity as UserEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub title: String,
    pub source: String,
    pub user_id: Uuid,
    pub text: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserEntity,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::UserEntity => Entity::belongs_to(UserEntity).into(),
        }
    }
}

impl Related<UserEntity> for Entity {
    fn to() -> RelationDef {
        Relation::UserEntity.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            title: ActiveValue::Set("".to_string()),
            source: ActiveValue::Set("".to_string()),
            user_id: ActiveValue::Set(Uuid::new_v4()),
            text: ActiveValue::Set("".to_string()),
            is_read: ActiveValue::Set(false),
            created_at: ActiveValue::Set(Utc::now()),
        }
    }
}

impl From<NotificationDTO> for Model {
    fn from(dto: NotificationDTO) -> Self {
        Self {
            id: dto.id,
            title: dto.title,
            source: dto.source,
            user_id: dto.user_id,
            text: dto.text,
            is_read: dto.is_read,
            created_at: dto.created_at,
        }
    }
}

impl From<CreateNotificationDTO> for Model {
    fn from(dto: CreateNotificationDTO) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: dto.title,
            source: dto.source,
            user_id: dto.user_id,
            text: dto.text,
            is_read: false,
            created_at: Utc::now(),
        }
    }
}
