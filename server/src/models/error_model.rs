use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::namespace_model::Entity as NamespaceEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "errors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub status_code: i16,
    pub user_affected: String,
    pub path: String,
    pub line: i32,
    pub message: String,
    pub stack_trace: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::models::namespace_model::Entity")]
    NamespaceEntity
}


impl Related<NamespaceEntity> for Entity {
    fn to() -> RelationDef {
        Relation::NamespaceEntity.def()
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
