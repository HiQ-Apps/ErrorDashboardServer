use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use shared_types::error_dtos::ErrorDTO;
use crate::models::namespace_model::Entity as NamespaceEntity;


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "errors")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub user_affected: String,
    pub path: String,
    pub line: i32,
    pub message: String,
    pub stack_trace: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[sea_orm(belongs_to = NamespaceEntity, from = "Column::NamespaceId", to = NamespaceModel::Column::Id)]
    pub namespace_id: Uuid,
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
    pub fn new(namespace_id: Uuid, namespace: Model) -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            namespace_id: ActiveValue::Set(namespace_id),  
            user_affected: ActiveValue::Set(namespace.user_affected),
            path: ActiveValue::Set(namespace.path),
            line: ActiveValue::Set(namespace.line),
            message: ActiveValue::Set(namespace.message),
            stack_trace: ActiveValue::Set(namespace.stack_trace),
            resolved: ActiveValue::Set(false),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now()),
        }
    }

    pub fn update_values(&mut self, new_values: Model) {
        self.user_affected = ActiveValue::Set(new_values.user_affected);
        self.path = ActiveValue::Set(new_values.path);
        self.line = ActiveValue::Set(new_values.line);
        self.message = ActiveValue::Set(new_values.message);
        self.stack_trace = ActiveValue::Set(new_values.stack_trace);
        self.resolved = ActiveValue::Set(new_values.resolved);
        self.updated_at = ActiveValue::Set(new_values.updated_at);
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl From<ErrorDTO> for Model {
    fn from(dto: ErrorDTO) -> Self {
        Self {
            id: dto.id,
            user_affected: dto.user_affected,
            path: dto.path,
            line: dto.line,
            message: dto.message,
            stack_trace: dto.stack_trace,
            resolved: dto.resolved,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            namespace_id: dto.namespace_id,
        }
    }
}
