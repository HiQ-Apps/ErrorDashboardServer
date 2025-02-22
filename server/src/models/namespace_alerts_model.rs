use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::namespace_model::Entity as NamespaceEntity;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "namespace_alerts")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub namespace_id: Uuid,
    pub alert_method: String,
    pub discord_channel_id: Option<String>,
    pub triggered: bool,
    pub path: Option<String>,
    pub line: Option<i32>,
    pub message: Option<String>,
    pub stack_trace: Option<String>,
    pub count_threshold: Option<i32>,
    pub time_window: Option<i64>,
    pub unresolved_time_threshold: Option<i64>,
    pub rate_threshold: Option<i32>,
    pub rate_time_window: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Namespace,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Namespace => Entity::belongs_to(NamespaceEntity)
                .from(Column::NamespaceId)
                .to(<NamespaceEntity as EntityTrait>::Column::Id)
                .into(),
        }
    }
}

impl Related<NamespaceEntity> for Entity {
    fn to() -> RelationDef {
        Relation::Namespace.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            namespace_id: ActiveValue::Set(Uuid::new_v4()),
            alert_method: ActiveValue::Set(String::new()),
            discord_channel_id: ActiveValue::Set(None),
            triggered: ActiveValue::Set(false),
            path: ActiveValue::Set(None),
            line: ActiveValue::Set(None),
            message: ActiveValue::Set(None),
            stack_trace: ActiveValue::Set(None),
            count_threshold: ActiveValue::Set(None),
            time_window: ActiveValue::Set(None),
            unresolved_time_threshold: ActiveValue::Set(None),
            rate_threshold: ActiveValue::Set(None),
            rate_time_window: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now()),
        }
    }
}
