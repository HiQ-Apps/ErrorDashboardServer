use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::error_model::Entity as ErrorEntity;
use crate::models::user_namespace_junction_model::Entity as UserNamespaceJunctionEntity;


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "namespaces")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub active: bool,
    pub service_name: String,
    pub environment_type: String,
    pub client_id: Uuid,
    pub client_secret: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ErrorEntity,
    UserNamespaceJunction,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ErrorEntity => Entity::has_many(ErrorEntity).into(),
            Self::UserNamespaceJunction => Entity::has_many(UserNamespaceJunctionEntity)
                .from(Column::Id)
                .to(<UserNamespaceJunctionEntity as EntityTrait>::Column::Id)
                .into(),
        }
    }
}

impl Related<ErrorEntity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorEntity.def()
    }
}

impl Related<UserNamespaceJunctionEntity> for Entity {
    fn to() -> RelationDef {
        Relation::UserNamespaceJunction.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::Set(Uuid::new_v4()),
            active: ActiveValue::Set(false),
            service_name: ActiveValue::Set(String::new()),
            environment_type: ActiveValue::Set(String::new()),
            client_id: ActiveValue::Set(Uuid::new_v4()),
            client_secret: ActiveValue::Set(Uuid::new_v4()),
            created_at: ActiveValue::Set(Utc::now()),
            updated_at: ActiveValue::Set(Utc::now()),
        }
    }
}
