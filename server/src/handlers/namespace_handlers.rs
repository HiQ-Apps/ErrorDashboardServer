use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use shared_types::error_dtos::AggregatedResult;
use shared_types::extra_dtos::{PaginationParams, ErrorQueryParams};
use shared_types::namespace_dtos::{CreateNamespaceDTO, UpdateNamespaceDTO};
use crate::managers::namespace_manager::NamespaceServer;
use crate::handlers::ws_handlers::WsNamespaceSession;
use crate::services::namespace_services::NamespaceService;
use crate::shared::utils::errors::{ServerError, ExternalError};
use crate::shared::utils::jwt::extract_user_id_from_jwt_header;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserId {
    pub user_id: Uuid
}

pub struct NamespaceHandler;

impl NamespaceHandler {

    pub async fn create_namespace(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
        new_namespace: web::Json<CreateNamespaceDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;

        let CreateNamespaceDTO { service_name, environment_type } = new_namespace.into_inner();
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
        user_id: web::Path<Uuid>,
        pagination: web::Query<PaginationParams>,
    ) -> Result<HttpResponse, ServerError> {
        let offset = pagination.offset;
        let limit = pagination.limit;
        match namespace_services.get_namespaces_by_user_id(*user_id, offset, limit).await {
            Ok(namespaces) => Ok(HttpResponse::Ok().json(namespaces)),
            Err(err) => Err(err)
        }
    }
    
    pub async fn update_namespace(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
        update_namespace_json: web::Json<UpdateNamespaceDTO>,
    ) -> Result<HttpResponse, ServerError> {

        let secret_key = &config.secret_key;
        let headers = req.headers();
        let user_id = extract_user_id_from_jwt_header(headers, secret_key)?;

        let update_namespace_dto = update_namespace_json.into_inner();
        match namespace_services.update_namespace(user_id, update_namespace_dto).await {
            Ok(updated_namespace) => Ok(HttpResponse::Ok().json(updated_namespace)),
            Err(err) => Err(err)
        }
    }

    pub async fn delete_namespace(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        let secret_key = &config.secret_key;
        let headers = req.headers();
        let user_id = extract_user_id_from_jwt_header(headers, secret_key)?;

        match namespace_services.delete_namespace(*namespace_id, user_id).await {
            Ok(id) => Ok(HttpResponse::Ok().json(id)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_errors_by_namespace_with_pagination(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
        query_params: web::Query<ErrorQueryParams>,
    ) -> Result<HttpResponse, ServerError> {
        let group_by = query_params.group_by.clone().unwrap_or_else(|| "message".to_string());
        let result = namespace_services.get_errors_by_namespace_with_pagination(
            *namespace_id,
            group_by,
            query_params.offset as usize,
            query_params.limit as usize,
        ).await?;
        match result {
            AggregatedResult::ByTags(tag) => Ok(HttpResponse::Ok().json(tag)),
            AggregatedResult::ByLine(line) => Ok(HttpResponse::Ok().json(line)),
             AggregatedResult::ByMessage(message) => Ok(HttpResponse::Ok().json(message)),
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
