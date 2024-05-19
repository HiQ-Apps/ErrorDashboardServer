use actix::{Actor, StreamHandler};
use actix_web::{web, HttpResponse, Result};
use actix_web_actors::ws;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;


use crate::services::error_services::ErrorService;
use crate::shared::utils::errors::ServerError;
use shared_types::error_dtos::CreateErrorDto;

pub struct ErrorHandler;

impl ErrorHandler {
    pub async fn create_error(
        error_services: web::Data<Arc<ErrorService>>,
        new_error: web::Json<CreateErrorDto>,
    ) -> Result<HttpResponse, ServerError> {
        let error_dto = new_error.into_inner();
        
        match error_services.create_error(error_dto).await {
            Ok(id) => Ok(HttpResponse::Ok().json(id)),
            Err(err) => Err(err)
        }
    }
}