use actix_web::Result;
use sea_orm::{
    entity::prelude::*, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, JoinType,
    QueryOrder, QuerySelect,
};
use std::sync::Arc;

use crate::config::Config;
use crate::models::feature_request_model::{
    ActiveModel, Entity as FeatureRequestEntity, Model as FeatureRequestModel,
};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError};
use shared_types::feature_request_dtos::{
    CreateFeatureRequestDTO, FeatureRequestDTO, UpdateFeatureRequestStatusDTO,
};

pub struct FeatureRequestService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}

impl FeatureRequestService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self> {
        Ok(Self { db, configs })
    }

    pub async fn create_feature_request(
        &self,
        feature_request: CreateFeatureRequestDTO,
    ) -> Result<(), ServerError> {
        let db = &*self.db;
        let create_feature_request: ActiveModel = feature_request.into();
        FeatureRequestEntity::insert(create_feature_request)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        Ok(())
    }

    pub async fn get_all_feature_requests(&self) -> Result<Vec<FeatureRequestModel>, ServerError> {
        let db = &*self.db;

        let feature_request = FeatureRequestEntity::find()
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(feature_request)
    }

    pub async fn update_feature_status(
        &self,
        update_feature_status: UpdateFeatureRequestStatusDTO,
    ) -> Result<(), ServerError> {
        let db = &*self.db;

        let feature_request = FeatureRequestEntity::find()
            .filter(<FeatureRequestEntity as EntityTrait>::Column::Id.eq(update_feature_status.id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        match feature_request {
            Some(mut feature_request) => {
                feature_request.status = update_feature_status.status;
                let active_feature = feature_request.into_active_model();
                active_feature
                    .update(db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
            }
            None => return Err(ServerError::QueryError(QueryError::FeatureRequestNotFound)),
        }

        Ok(())
    }

    pub async fn delete_feature_request(&self, id: i32) -> Result<(), ServerError> {
        let db = &*self.db;

        let feature_request = FeatureRequestEntity::find()
            .filter(<FeatureRequestEntity as EntityTrait>::Column::Id.eq(id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        match feature_request {
            Some(feature_request) => {
                feature_request
                    .delete(db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
            }
            None => return Err(ServerError::QueryError(QueryError::FeatureRequestNotFound)),
        }

        Ok(())
    }
}
