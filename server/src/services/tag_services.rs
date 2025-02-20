use sea_orm::{entity::prelude::*, DatabaseConnection, EntityTrait};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::models::error_tag_model::{ActiveModel as ActiveTagModel, Entity as TagEntity};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError};
use shared_types::tag_dtos::{CreateTagDTO, TagDTO};

pub struct TagService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl TagService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_tag(&self, tag: CreateTagDTO) -> Result<(), ServerError> {
        let db = &*self.db;
        let create_tag: ActiveTagModel = tag.into();
        TagEntity::insert(create_tag)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(())
    }

    pub async fn delete_tag(&self, tag_id: Uuid) -> Result<(), ServerError> {
        let db = &*self.db;
        let found_tag = TagEntity::find()
            .filter(<TagEntity as EntityTrait>::Column::Id.eq(tag_id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        match found_tag {
            Some(tag) => {
                tag.delete(db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
                Ok(())
            }
            None => Err(ServerError::QueryError(QueryError::InvalidTag)),
        }
    }

    pub async fn get_tags_by_error_id(&self, error_id: Uuid) -> Result<Vec<TagDTO>, ServerError> {
        let db = &*self.db;
        let tags = TagEntity::find()
            .filter(<TagEntity as EntityTrait>::Column::ErrorId.eq(error_id))
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(tags
            .into_iter()
            .map(|tag| TagDTO {
                id: tag.id,
                tag_key: tag.tag_key,
                tag_value: tag.tag_value,
                error_id: tag.error_id,
                tag_color: tag.tag_color,
            })
            .collect())
    }
}
