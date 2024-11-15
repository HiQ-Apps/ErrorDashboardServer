use actix_web::{web, HttpResponse, HttpRequest, Result, cookie::{Cookie, SameSite, time::Duration}};
use std::sync::Arc;

use crate::shared::utils::errors::ServerError;
use crate::services::{UserService, NamespaceService};

pub struct AdminHandler;

impl AdminHandler {
    pub async fn get_all_user_data(
        user_services: web::Data<Arc<UserService>>,
    ) -> Result<HttpResponse, ServerError> {
        match user_services.get_all_users().await {
            Ok(user_data) => Ok(HttpResponse::Ok().json(user_data)),
            Err(err) => Err(err)
        }
    }
    pub async fn get_all_namespaces(
        namespace_services: web::Data<Arc<NamespaceService>>,
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.get_all_namespaces().await {
            Ok(namespaces) => Ok(HttpResponse::Ok().json(namespaces)),
            Err(err) => Err(err)
        }
    }

}

