use actix_web::Result;
use sea_orm::{
    entity::prelude::*, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, JoinType,
    QueryOrder, QuerySelect,
};
use std::sync::Arc;

use crate::config::Config;
use crate::models::bug_report_model::{
    ActiveModel, Entity as BugReportEntity, Model as BugReportModel,
};
use crate::shared::utils::errors::{ExternalError, QueryError, ServerError};
use shared_types::bug_report_dtos::{BugReportDTO, CreateBugReportDTO, UpdateBugReportStatusDTO};

pub struct BugReportService {
    pub db: Arc<DatabaseConnection>,
    pub configs: Arc<Config>,
}
impl BugReportService {
    pub fn new(db: Arc<DatabaseConnection>, configs: Arc<Config>) -> Result<Self> {
        Ok(Self { db, configs })
    }

    pub async fn create_bug_report(
        &self,
        bug_report: CreateBugReportDTO,
    ) -> Result<(), ServerError> {
        let db = &*self.db;
        let create_bug_report: ActiveModel = bug_report.into();
        BugReportEntity::insert(create_bug_report)
            .exec(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
        Ok(())
    }

    pub async fn get_all_bug_reports(&self) -> Result<Vec<BugReportModel>, ServerError> {
        let db = &*self.db;

        let bug_report = BugReportEntity::find()
            .all(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        Ok(bug_report)
    }

    pub async fn update_bug_status(
        &self,
        update_bug_status: UpdateBugReportStatusDTO,
    ) -> Result<(), ServerError> {
        let db = &*self.db;

        let bug_report = BugReportEntity::find()
            .filter(<BugReportEntity as EntityTrait>::Column::Id.eq(update_bug_status.id))
            .one(db)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;

        match bug_report {
            Some(mut bug_report) => {
                bug_report.status = update_bug_status.status;
                let active_bug_report = bug_report.into_active_model();
                active_bug_report
                    .update(db)
                    .await
                    .map_err(|err| ServerError::ExternalError(ExternalError::DB(err)))?;
                Ok(())
            }
            None => Err(ServerError::QueryError(QueryError::NotFound)),
        }
    }
}
