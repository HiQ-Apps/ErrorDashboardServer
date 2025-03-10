use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, prelude::async_trait::async_trait, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::refresh_token_model::Entity as RefreshTokenEntity;
use crate::models::user_profile_model::Entity as UserProfileEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password: Option<String>,
    pub user_profile_id: Uuid,
    pub o_auth_provider: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    RefreshTokenEntity,
    UserProfileEntity,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::UserProfileEntity => {
                Entity::belongs_to(UserProfileEntity)
                    .from(Column::UserProfileId)
                    .to(<UserProfileEntity as EntityTrait>::Column::Id)
                    .into()
            }
            Self::RefreshTokenEntity => Entity::has_many(RefreshTokenEntity).into(),
        }
    }
}

impl Related<UserProfileEntity> for Entity {
    fn to() -> RelationDef {
        Entity::belongs_to(UserProfileEntity)
            .from(Column::UserProfileId)
            .to(<UserProfileEntity as EntityTrait>::Column::Id)
            .into()
    }
}


impl Related<RefreshTokenEntity> for Entity {
    fn to() -> RelationDef {
        Relation::RefreshTokenEntity.def()
    }
}

impl ActiveModel {
    pub fn update_values(&mut self, new_values: Model) {
        self.username = ActiveValue::Set(new_values.username);
        self.email = ActiveValue::Set(new_values.email);
        self.password = ActiveValue::Set(new_values.password);
        self.verified = ActiveValue::Set(new_values.verified);
        self.created_at = ActiveValue::Set(new_values.created_at);
        self.updated_at = ActiveValue::Set(new_values.updated_at);
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_profile_id: ActiveValue::Set(Uuid::new_v4()),
            username: ActiveValue::Set(String::new()),
            email: ActiveValue::Set(String::new()),
            password: ActiveValue::Set(Some(String::new())),
            verified: ActiveValue::Set(false),
            o_auth_provider: ActiveValue::Set(String::new()),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now()),
        }
    }
}
