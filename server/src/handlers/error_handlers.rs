use actix::Addr;
use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use crate::managers::namespace_manager::NamespaceServer;
use crate::handlers::ws_handlers::NewError;
use crate::services::error_services::ErrorService;
use crate::shared::utils::errors::ServerError;
use shared_types::{{extra_dtos::TimeParams, error_dtos::{CreateErrorRequest, UpdateErrorDTO}}};



pub struct ErrorHandler;

impl ErrorHandler {
    pub async fn create_error(
        error_services: web::Data<Arc<ErrorService>>,
        namespace_manager: web::Data<Addr<NamespaceServer>>,
        new_error: web::Json<CreateErrorRequest>,
    ) -> Result<HttpResponse, ServerError> {
        let error_dto = new_error.into_inner();
        
        match error_services.create_error(error_dto).await {
            Ok(error_dto) => {
                namespace_manager.do_send(NewError(error_dto.clone()));
                Ok(HttpResponse::Ok().finish())
            },
            Err(err) => Err(err)
        }
    }

    pub async fn get_error_by_id(
        error_services: web::Data<Arc<ErrorService>>,
        error_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match error_services.get_error_by_id(error_id.into_inner()).await {
            Ok(error_dto) => Ok(HttpResponse::Ok().json(error_dto)),
            Err(err) => Err(err)
        }
    }

    pub async fn update_error(
        error_services: web::Data<Arc<ErrorService>>,
        update_error: web::Json<UpdateErrorDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let error_dto = update_error.into_inner();
     
        match error_services.update_error(error_dto).await {
            Ok(error_dto) => Ok(HttpResponse::Ok().json(error_dto)),
            Err(err) => Err(err)
        }
    }


    pub async fn get_aggregate_errors_by_date(
        error_services: web::Data<Arc<ErrorService>>,
        namespace_id: web::Path<Uuid>,
        time_params_query: web::Query<TimeParams>,
    ) -> Result<HttpResponse, ServerError> {
        let time_params = time_params_query.into_inner();
        let namespace_id = namespace_id.into_inner();
        let time_interval_minutes = time_params.time_interval_minutes;
        let start_time = time_params.start_time;
        let timezone = time_params.timezone;
        match error_services.get_aggregate_errors_by_date(namespace_id, start_time, time_interval_minutes, timezone).await {
            Ok(errors) => Ok(HttpResponse::Ok().json(errors)),
            Err(err) => Err(err)
        }
    }
}
