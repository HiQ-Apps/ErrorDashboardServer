use async_trait::async_trait;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::namespace_alerts_model::Entity as NamespaceAlertEntity;
use crate::models::user_model::Entity as UserEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "namespace_alert_user_junction")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub namespace_alert_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    NamespaceAlertEntity,
    UserEntity,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::NamespaceAlertEntity => Entity::belongs_to(NamespaceAlertEntity)
                .from(Column::NamespaceAlertId)
                .to(<NamespaceAlertEntity as EntityTrait>::Column::Id)
                .into(),
            Self::UserEntity => Entity::belongs_to(UserEntity)
                .from(Column::UserId)
                .to(<UserEntity as EntityTrait>::Column::Id)
                .into(),
        }
    }
}

impl Related<NamespaceAlertEntity> for Entity {
    fn to() -> RelationDef {
        Relation::NamespaceAlertEntity.def()
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
            id: ActiveValue::NotSet,
            namespace_alert_id: ActiveValue::NotSet,
            user_id: ActiveValue::NotSet,
        }
    }
}
