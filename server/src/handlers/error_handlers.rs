use actix::Addr;
use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use crate::managers::namespace_manager::NamespaceServer;
use crate::handlers::ws_handlers::NewError;
use crate::services::error_services::ErrorService;
use crate::shared::utils::errors::ServerError;
use shared_types::error_dtos::{CreateErrorDto, UpdateErrorDto};


pub struct ErrorHandler;

impl ErrorHandler {
    pub async fn create_error(
        error_services: web::Data<Arc<ErrorService>>,
        namespace_manager: web::Data<Addr<NamespaceServer>>,
        new_error: web::Json<CreateErrorDto>,
    ) -> Result<HttpResponse, ServerError> {
        let error_dto = new_error.into_inner();
        
        match error_services.create_error(error_dto).await {
            Ok(error_dto) => {
                namespace_manager.do_send(NewError(error_dto.clone()));
                Ok(HttpResponse::Ok().json(error_dto))
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
        update_error: web::Json<UpdateErrorDto>,
    ) -> Result<HttpResponse, ServerError> {
        let error_dto = update_error.into_inner();
     
        match error_services.update_error(error_dto).await {
            Ok(error_dto) => Ok(HttpResponse::Ok().json(error_dto)),
            Err(err) => Err(err)
        }
    }

}
