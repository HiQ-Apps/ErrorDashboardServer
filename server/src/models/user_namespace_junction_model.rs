use chrono::NaiveDateTime;
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::namespace_model::Entity as NamespaceEntityEntity;
use crate::models::user_model::Entity as UserEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_namespace_junction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::models::namespace::Entity", from_foreign_key = "namespace_id", to_primary_key = "id")]
    NamespaceEntity,
    #[sea_orm(belongs_to = "crate::models::user::Entity", from_foreign_key = "user_id", to_primary_key = "id")]
    UserEntity,
}

impl Related<NamespaceEntity> for Entity {
    fn to() -> RelationDef {
        Relation::NamespaceEntity.def()
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
            ..ActiveModelTrait::default()
        }
    }
}

