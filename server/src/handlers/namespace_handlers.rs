use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use shared_types::namespace_dtos::CreateNamespaceDto;
use crate::services::namespace_services::NamespaceService;
use crate::shared::utils::errors::ServerError;

pub struct NamespaceHandler;

impl NamespaceHandler {

    pub async fn create_namespace(
        namespace_services: web::Data<Arc<NamespaceService>>,
        new_namespace: web::Json<CreateNamespaceDto>,
    ) -> Result<HttpResponse, ServerError> {
        let CreateNamespaceDto { service_name, environment_type, user_id } = new_namespace.into_inner();
        match namespace_services.create_namespace(user_id, service_name, environment_type).await {
            Ok(id) => Ok(HttpResponse::Ok().json(id)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_namespace_by_id(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.get_namespace_by_id(*namespace_id).await {
            Ok(namespace) => Ok(HttpResponse::Ok().json(namespace)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_namespaces_by_user_id(
        namespace_services: web::Data<Arc<NamespaceService>>,
        user_id: web::Path<Uuid>
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.get_namespaces_by_user_id(*user_id).await {
            Ok(namespaces) => Ok(HttpResponse::Ok().json(namespaces)),
            Err(err) => Err(err)
        }
    }
    
    pub async fn update_namespace(
        namespace_services: web::Data<Arc<NamespaceService>>,
    ) -> Result<HttpResponse, ServerError> {
        Ok(HttpResponse::Ok().json("Not implemented"))
    }

    pub async fn delete_namespace(
        namespace_services: web::Data<Arc<NamespaceService>>,
    ) -> Result<HttpResponse, ServerError> {
        Ok(HttpResponse::Ok().json("Not implemented"))
    }
}
