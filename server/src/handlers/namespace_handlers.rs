use actix_web::{web, HttpResponse, Result};
use chrono::offset;
use std::sync::Arc;
use uuid::Uuid;

use shared_types::namespace_dtos::{CreateNamespaceDto, UpdateNamespaceDto};
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
        update_namespace_json: web::Json<UpdateNamespaceDto>,
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.update_namespace(update_namespace_json.into_inner()).await {
            Ok(updated_namespace) => Ok(HttpResponse::Ok().json(updated_namespace)),
            Err(err) => Err(err)
        }
    }

    pub async fn delete_namespace(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
        user_id: web::Json<Uuid>
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.delete_namespace(*namespace_id, *user_id).await {
            Ok(id) => Ok(HttpResponse::Ok().json(id)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_errors_by_namespace_with_pagination(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
        offset: web::Query<u64>,
        limit: web::Query<u64>,
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.get_errors_by_namespace_with_pagination(*namespace_id, *offset, *limit).await {
            Ok(errors) => Ok(HttpResponse::Ok().json(errors)),
            Err(err) => Err(err)
        }
    }
}
