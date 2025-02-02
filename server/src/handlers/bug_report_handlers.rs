use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;

use crate::services::bug_report_services::BugReportService;
use crate::shared::utils::errors::ServerError;
use shared_types::bug_report_dtos::{CreateBugReportDTO, UpdateBugReportStatusDTO};

pub struct BugReportHandler;

impl BugReportHandler {
    pub async fn create_bug_report(
        bug_report_services: web::Data<Arc<BugReportService>>,
        create_bug_report: web::Json<CreateBugReportDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let create_bug_report = create_bug_report.into_inner();
        match bug_report_services.create_bug_report(create_bug_report).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn get_all_bug_reports(
        bug_report_services: web::Data<Arc<BugReportService>>,
    ) -> Result<HttpResponse, ServerError> {
        match bug_report_services.get_all_bug_reports().await {
            Ok(bug_reports) => Ok(HttpResponse::Ok().json(bug_reports)),
            Err(err) => Err(err),
        }
    }

    pub async fn update_bug_status(
        bug_report_services: web::Data<Arc<BugReportService>>,
        update_bug_status: web::Json<UpdateBugReportStatusDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let update_bug_status = update_bug_status.into_inner();
        match bug_report_services.update_bug_status(update_bug_status).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

}