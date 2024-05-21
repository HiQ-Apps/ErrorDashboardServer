use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::managers::namespace_manager::NamespaceServer;
use crate::handlers::ws_handlers::WsNamespaceSession;
use shared_types::namespace_dtos::{CreateNamespaceDto, UpdateNamespaceDto};
use shared_types::extra_dtos::PaginationParams;
use crate::services::namespace_services::NamespaceService;
use crate::shared::utils::errors::{ServerError, ExternalError};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserId {
    pub user_id: Uuid
}

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
        println!("{:?}", update_namespace_json);
        let update_namespace_dto = update_namespace_json.into_inner();
        println!("{:?}", update_namespace_dto);
        match namespace_services.update_namespace(update_namespace_dto).await {
            Ok(updated_namespace) => Ok(HttpResponse::Ok().json(updated_namespace)),
            Err(err) => Err(err)
        }
    }

    pub async fn delete_namespace(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
        user_id: web::Json<UserId>
    ) -> Result<HttpResponse, ServerError> {
        match namespace_services.delete_namespace(*namespace_id, user_id.user_id).await {
            Ok(id) => Ok(HttpResponse::Ok().json(id)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_errors_by_namespace_with_pagination(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
        pagination: web::Query<PaginationParams>,
    ) -> Result<HttpResponse, ServerError> {
        let offset = pagination.offset;
        let limit = pagination.limit;

        match namespace_services.get_errors_by_namespace_with_pagination(*namespace_id, offset, limit).await {
            Ok(errors) => Ok(HttpResponse::Ok().json(errors)),
            Err(err) => Err(err)
        }
    }

    pub async fn ws_index(
        req: HttpRequest,
        stream: web::Payload,
        namespace_id: web::Path<Uuid>,
        manager_addr: web::Data<Addr<NamespaceServer>>,
    ) -> Result<HttpResponse, ServerError> {
        let namespace_id = namespace_id.into_inner();
        let manager_addr = manager_addr.get_ref().clone();

        let session = WsNamespaceSession {
            namespace_id,
            addr: manager_addr.clone(),
        };
        ws::start(session, &req, stream).map_err(|err| ServerError::ExternalError(ExternalError::Actix(err)))
    }
}
