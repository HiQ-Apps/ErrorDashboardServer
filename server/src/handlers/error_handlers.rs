use actix::{Addr, Actor, StreamHandler};
use actix_web::{web, HttpResponse, Result};
use actix_web_actors::ws;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::managers::namespace_manager::NamespaceServer;
use crate::handlers::ws_handlers::NewError;
use crate::services::error_services::ErrorService;
use crate::shared::utils::errors::ServerError;
use shared_types::error_dtos::CreateErrorDto;


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
}