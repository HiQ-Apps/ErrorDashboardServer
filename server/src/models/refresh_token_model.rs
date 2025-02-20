use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sea_orm::entity::RelationDef;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user_model::Entity as UserEntity;

// DeriveEntityModel generates the ActiveModel, Model and ActiveModelTrait
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub token: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub issuer: String,
    pub audience: String,
    pub revoked: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserEntity,
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

// On update, update current ActiveModel with new values
impl ActiveModel {
    pub fn update_values(&mut self, new_values: Model) {
        self.user_id = ActiveValue::Set(new_values.user_id);
        self.token = ActiveValue::Set(new_values.token);
        self.issued_at = ActiveValue::Set(new_values.issued_at);
        self.expires_at = ActiveValue::Set(new_values.expires_at);
        self.issuer = ActiveValue::Set(new_values.issuer);
        self.audience = ActiveValue::Set(new_values.audience);
        self.revoked = ActiveValue::Set(new_values.revoked);
    }
}

// Provide a default implementation of the ActiveModelBehavior trait for the ActiveModel struct
#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Unchanged(None),
            token: ActiveValue::Set(String::new()),
            issued_at: ActiveValue::Set(Utc::now()),
            expires_at: ActiveValue::Set(Utc::now() + Duration::days(1)),
            issuer: ActiveValue::Set("default_issuer".to_string()),
            audience: ActiveValue::Set("default_audience".to_string()),
            revoked: ActiveValue::Set(false),
        }
    }
}
