use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::user_dtos::{PasswordDTO, ResetPasswordPath, ResetPasswordRequestDTO, UpdateUserProfileDTO, UserLoginDTO};
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

    pub async fn verify_user(
        user_services: web::Data<Arc<UserService>>,
        user_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match user_services.verify_user(user_id.into_inner()).await {
            Ok(()) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
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

    pub async fn forgot_password(
        user_services: web::Data<Arc<UserService>>,
        email: web::Json<ResetPasswordRequestDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let email = email.into_inner();
        match user_services.forgot_password(email).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }

    pub async fn update_password(
        user_services: web::Data<Arc<UserService>>,
        reset_password: web::Json<PasswordDTO>,
        user_params: web::Path<ResetPasswordPath>
    ) -> Result<HttpResponse, ServerError> {
        let updated_password = reset_password.into_inner();
        let user_params = user_params.into_inner();
        let password = updated_password.password;
        let user_id = user_params.id;
        let email = user_params.email;

        match user_services.update_password(user_id, email, password).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
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

