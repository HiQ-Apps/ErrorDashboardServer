use async_trait::async_trait;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::namespace_model::Entity as NamespaceEntity;
use crate::models::user_model::Entity as UserEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_namespace_junction")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    NamespaceEntity,
    UserEntity,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::NamespaceEntity => Entity::belongs_to(NamespaceEntity)
                .from(<Entity as sea_orm::EntityTrait>::Column::NamespaceId)
                .to(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id)
                .into(),
            Self::UserEntity => Entity::belongs_to(UserEntity)
                .from(<Entity as sea_orm::EntityTrait>::Column::UserId)
                .to(<UserEntity as sea_orm::EntityTrait>::Column::Id)
                .into(),
        }
    }
}

impl Related<NamespaceEntity> for Entity {
    fn to() -> RelationDef {
        Entity::belongs_to(NamespaceEntity)
            .from(Column::NamespaceId)
            .to(<NamespaceEntity as sea_orm::EntityTrait>::Column::Id)
            .into()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            namespace_id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(Uuid::new_v4()),
            role: ActiveValue::Set(String::new()),
        }
    }
}
