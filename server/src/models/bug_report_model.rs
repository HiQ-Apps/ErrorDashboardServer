use sea_orm::{entity::prelude::*, ActiveValue};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use shared_types::bug_report_dtos::CreateBugReportDTO;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bug_report")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Integer", auto_increment = true)]
    pub id: i32,
    pub issue: String,
    pub description: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        unreachable!();
    }
}


#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            issue: ActiveValue::NotSet,
            description: ActiveValue::NotSet,
            status: ActiveValue::NotSet,
            created_at: ActiveValue::NotSet,
        }
    }
}

impl From<CreateBugReportDTO> for ActiveModel {
    fn from(dto: CreateBugReportDTO) -> Self {
        Self {
            id: ActiveValue::NotSet,
            issue: ActiveValue::Set(dto.issue),
            description: ActiveValue::Set(dto.description),
            status: ActiveValue::Set("Pending".to_string()),
            created_at: ActiveValue::Set(Utc::now()),
        }
    }
}

