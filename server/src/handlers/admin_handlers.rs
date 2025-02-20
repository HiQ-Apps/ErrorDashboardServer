use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_types::auth_dtos::VerifyUserDTO;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::services::{NamespaceService, UserService};
use crate::shared::utils::jwt::extract_user_id_from_jwt_header;
use crate::{
    services::AuthService,
    shared::utils::errors::{QueryError, ServerError},
};

pub struct AdminHandler;

impl AdminHandler {
    pub async fn get_all_user_data(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        user_services: web::Data<Arc<UserService>>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;

        let role = user_services.get_user_role_by_user_id(user_id).await?;

        if role != "admin" {
            return Err(ServerError::QueryError(QueryError::InvalidRole));
        }

        match user_services.get_all_users().await {
            Ok(user_data) => Ok(HttpResponse::Ok().json(user_data)),
            Err(err) => Err(err),
        }
    }
    pub async fn get_all_namespaces(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        user_services: web::Data<Arc<UserService>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;

        let role = user_services.get_user_role_by_user_id(user_id).await?;

        if role != "admin" {
            return Err(ServerError::QueryError(QueryError::InvalidRole));
        }

        match namespace_services.get_all_namespaces().await {
            Ok(namespaces) => Ok(HttpResponse::Ok().json(namespaces)),
            Err(err) => Err(err),
        }
    }

    pub async fn verify_admin(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        user_services: web::Data<Arc<UserService>>,
        auth_services: web::Data<Arc<AuthService>>,
        password: web::Json<VerifyUserDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;

        let role = user_services.get_user_role_by_user_id(user_id).await?;

        if role != "admin" {
            return Err(ServerError::QueryError(QueryError::InvalidRole));
        }

        let VerifyUserDTO { password } = password.into_inner();

        match auth_services.verify_user(user_id, password).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }
}
