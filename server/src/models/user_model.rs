use chrono::NaiveDateTime;
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::refresh_token_model::Entity as RefreshTokenEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::models::refresh_token_model::Entity")]
    RefreshTokenEntity,
}

impl Related<RefreshTokenEntity> for Entity {
    fn to() -> RelationDef {
        Relation::RefreshTokenEntity.def()
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
