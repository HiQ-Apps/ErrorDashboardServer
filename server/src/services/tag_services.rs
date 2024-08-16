use std::sync::Arc;
use sea_orm::{entity::prelude::*, EntityTrait, DatabaseConnection};
use uuid::Uuid;

use crate::config::Config;
use shared_types::tag_dtos::{CreateTagDTO, TagDTO};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError};
use crate::models::error_tag_model::{Entity as TagEntity, ActiveModel as ActiveTagModel};

pub struct TagService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>
}

impl TagService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self, ServerError> {
        Ok(Self { db, configs })
    }

    pub async fn create_tag(
        &self,
        tag: CreateTagDTO
    ) -> Result<(), ServerError> {
        let create_tag: ActiveTagModel = tag.into();
        TagEntity::insert(create_tag)
            .exec(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(())
    }

    pub async fn delete_tag(
        &self,
        tag_id: Uuid
    ) -> Result<(), ServerError> {
        let found_tag = TagEntity::find()
            .filter(<TagEntity as EntityTrait>::Column::Id.eq(tag_id))
            .one(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        match found_tag {
            Some(tag) => {
                tag.delete(&*self.db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
                Ok(())
            },
            None => Err(ServerError::QueryError(QueryError::InvalidTag))
        } 
    }

    pub async fn get_tags_by_error_id(
        &self,
        error_id: Uuid
    ) -> Result<Vec<TagDTO>, ServerError> {
        let tags = TagEntity::find()
            .filter(<TagEntity as EntityTrait>::Column::ErrorId.eq(error_id))
            .all(&*self.db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(tags.into_iter().map(|tag| TagDTO {
            id: tag.id,
            tag_key: tag.tag_key,
            tag_value: tag.tag_value,
            error_id: tag.error_id,
            tag_color: tag.tag_color
        }).collect())
    }

}
