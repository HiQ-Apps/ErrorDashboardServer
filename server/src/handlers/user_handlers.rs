use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::user_dtos::UpdateUserProfileDTO;
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


    pub async fn get_user_profile(
        user_services: web::Data<Arc<UserService>>,
        id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match user_services.get_user_profile(id.into_inner()).await {
            Ok(user_profile) => Ok(HttpResponse::Ok().json(user_profile)),
            Err(err) => Err(err)
        }
    }

    pub async fn update_user_profile(
        user_services: web::Data<Arc<UserService>>,
        user_id: web::Path<Uuid>,
        update_user_profile: web::Json<UpdateUserProfileDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let update_user_profile_dto = update_user_profile.into_inner();
        match user_services.update_user_profile(user_id.into_inner(), update_user_profile_dto).await {
            Ok(user_profile) => Ok(HttpResponse::Ok().json(user_profile)),
            Err(err) => Err(err)
        }
    }
}

