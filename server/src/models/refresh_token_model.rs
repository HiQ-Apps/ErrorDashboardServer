use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait};
use sea_orm::entity::RelationDef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user_model::Entity as UserEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "refresh_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub issuer: String,
    pub audience: String,
    pub revoked: bool
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserEntity,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::UserEntity => Entity::has_one(UserEntity).into(),
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
            ..ActiveModelTrait::default()
        }
    }
}
