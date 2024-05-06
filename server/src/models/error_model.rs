use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::namespace_model::Entity as NamespaceEntity;


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "errors")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
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

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    NamespaceEntity
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::NamespaceEntity => Entity::belongs_to(NamespaceEntity).into(),
        }
    }
}


impl Related<NamespaceEntity> for Entity {
    fn to() -> RelationDef {
        Relation::NamespaceEntity.def()
    }
}

impl ActiveModel {
    pub fn update_values(&mut self, new_values: Model) {
        self.status_code = ActiveValue::Set(new_values.status_code);
        self.user_affected = ActiveValue::Set(new_values.user_affected);
        self.path = ActiveValue::Set(new_values.path);
        self.line = ActiveValue::Set(new_values.line);
        self.message = ActiveValue::Set(new_values.message);
        self.stack_trace = ActiveValue::Set(new_values.stack_trace);
        self.resolved = ActiveValue::Set(new_values.resolved);
        self.created_at = ActiveValue::Set(new_values.created_at);
        self.updated_at = ActiveValue::Set(new_values.updated_at);
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            status_code: ActiveValue::Set(0),
            user_affected: ActiveValue::Set("".to_string()),
            path: ActiveValue::Set("".to_string()),
            line: ActiveValue::Set(0),
            message: ActiveValue::Set("".to_string()),
            stack_trace: ActiveValue::Set("".to_string()),
            resolved: ActiveValue::Set(false),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now())
        }
    }
}
