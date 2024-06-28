use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user_model::Entity as UserEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_profile")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserEntity
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

impl ActiveModel {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(Uuid::new_v4()),
            first_name: ActiveValue::Unchanged(None),
            last_name: ActiveValue::Unchanged(None),
            avatar_color: ActiveValue::Set(String::new()),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now()),
        }
    }
}

