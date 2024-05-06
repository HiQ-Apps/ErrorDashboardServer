use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::error_model::Entity as ErrorEntity;
use crate::models::user_namespace_junction_model::Entity as UserNamespaceJunctionEntity;


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "namespace")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub error_records: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ErrorEntity,
    UserNamespaceJunction,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ErrorEntity => Entity::has_many(ErrorEntity).into(),
            Self::UserNamespaceJunction => Entity::has_many(UserNamespaceJunctionEntity)
                .from(Column::Id)
                .to(<UserNamespaceJunctionEntity as sea_orm::EntityTrait>::Column::Id)
                .into(),
        }
    }
}

impl Related<ErrorEntity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorEntity.def()
    }
}

impl Related<UserNamespaceJunctionEntity> for Entity {
    fn to() -> RelationDef {
        Relation::UserNamespaceJunction.def()
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
