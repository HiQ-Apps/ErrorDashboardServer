use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::error_model::Entity as ErrorEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "namespace")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub error_records: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::models::error_model::Entity")]
    ErrorEntity,
}

impl Related<ErrorEntity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorEntity.def()
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
