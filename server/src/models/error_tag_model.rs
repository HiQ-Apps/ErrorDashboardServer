use async_trait::async_trait;
use sea_orm::entity::{prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::error_model::Entity as ErrorEntity;
use shared_types::tag_dtos::CreateTagDTO;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "error_tags")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub error_id: Uuid,
    pub tag_key: String,
    pub tag_value: String
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ErrorEntity
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ErrorEntity => Entity::belongs_to(ErrorEntity).into(),
        }
    }
}

impl Related<ErrorEntity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorEntity.def()
    }
}

impl ActiveModel {
    pub fn new(error_id: Uuid, tag_key: String, tag_value: String) -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            error_id: ActiveValue::Set(error_id),
            tag_key: ActiveValue::Set(tag_key),
            tag_value: ActiveValue::Set(tag_value)
        }
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}


impl From<CreateTagDTO> for ActiveModel {
    fn from(dto: CreateTagDTO) -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            error_id: ActiveValue::Set(dto.error_id),
            tag_key: ActiveValue::Set(dto.tag_key),
            tag_value: ActiveValue::Set(dto.tag_value)
        }
    }
}

