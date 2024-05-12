use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::utils::errors::ServerError;
use crate::services::UserService;

pub struct UserHandler;

impl UserHandler {
    pub async fn get_user(
        user_services: web::Data<Arc<UserService>>,
        user_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match user_services.get_user(user_id.into_inner()).await {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(err) => Err(err) 
        }
    }

    pub async fn delete_user(
        user_services: web::Data<Arc<UserService>>,
        user_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match user_services.delete_user(user_id.into_inner()).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }
}

