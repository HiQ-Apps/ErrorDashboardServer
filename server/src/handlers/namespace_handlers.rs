use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{self, Message};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::shared::utils::role::{get_weight, string_to_role, Permission, RoleRules};
use shared_types::error_dtos::AggregatedResult;
use shared_types::extra_dtos::{PaginationParams, ErrorQueryParams};
use shared_types::namespace_dtos::{CreateNamespaceDTO, InviteUserRequestDTO, UpdateNamespaceDTO, UpdateUserRoleRequestDTO};
use crate::managers::namespace_manager::NamespaceServer;
use crate::services::namespace_services::NamespaceService;
use crate::handlers::ws_handlers::namespace_error_ws_session;
use crate::shared::utils::errors::{ExternalError, QueryError, RequestError, ServerError};
use crate::shared::utils::jwt::extract_user_id_from_jwt_header;


#[derive(Debug, Serialize, Deserialize)]
pub struct UserId {
    pub user_id: Uuid
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveNamespacePathParams {
    pub user_id: Uuid,
    pub namespace_id: Uuid
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
        role_rules: web::Data<Arc<RoleRules>>
    ) -> Result<HttpResponse, ServerError> {
        let secret_key = &config.secret_key;
        let headers = req.headers();
        let user_id = extract_user_id_from_jwt_header(headers, secret_key)?;
        let role_rules = role_rules.as_ref().as_ref();

        let update_namespace_dto = update_namespace_json.into_inner();
        match namespace_services.update_namespace(user_id, update_namespace_dto, role_rules.clone()).await {
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
            Ok(()) => Ok(HttpResponse::Ok().finish()),
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

    pub async fn namespace_error_ws_session(
        req: HttpRequest,
        stream: web::Payload,
        namespace_id: web::Path<Uuid>,
        namespace_server: web::Data<Arc<NamespaceServer>>,
    ) -> Result<HttpResponse, ServerError> {
        let namespace_id = namespace_id.into_inner();
        let namespace_server = namespace_server.get_ref().clone();

        let (response, session, _msg_stream) = actix_ws::handle(&req, stream).map_err(|err| {
            ServerError::ExternalError(ExternalError::Actix(err))
        })?;

        actix_web::rt::spawn(namespace_error_ws_session(session, namespace_id, namespace_server));

        Ok(response)
    }

    pub async fn invite_user_to_namespace(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
        invite_user_request: web::Json<InviteUserRequestDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let invite_user_data = invite_user_request.into_inner();
        let namespace_id = namespace_id.into_inner();
        match namespace_services.invite_user_to_namespace(namespace_id, invite_user_data).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }

    pub async fn remove_user_from_namespace(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
        path_params: web::Path<RemoveNamespacePathParams>,
    ) -> Result<HttpResponse, ServerError>{
        let headers = req.headers();
        let secret_key = &config.secret_key;
        let deleter_id = extract_user_id_from_jwt_header(headers, secret_key)?;

        let path_params = path_params.into_inner();
        let namespace_id = path_params.namespace_id;
        let deletee_id = path_params.user_id;
        
        match namespace_services.check_namespace_membership(deleter_id, namespace_id).await {
            Ok(res) => {
                if !res {
                    return Err(ServerError::RequestError(RequestError::PermissionDenied));
                }
            },
            Err(err) => return Err(err)
        }

        match namespace_services.remove_user_from_namespace( deletee_id, namespace_id).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }

    pub async fn get_namespace_members(
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        let namespace_id = namespace_id.into_inner();
        match namespace_services.get_namespace_members(namespace_id).await {
            Ok(members) => Ok(HttpResponse::Ok().json(members)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_user_role_in_namespace(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
        namespace_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = &config.secret_key;
        let user_id = extract_user_id_from_jwt_header(headers, secret_key)?;

        let namespace_id = namespace_id.into_inner();
        match namespace_services.get_user_role_in_namespace(user_id, namespace_id).await {
            Ok(role) => Ok(HttpResponse::Ok().json(role)),
            Err(err) => Err(err)
        }
    }

    pub async fn update_user_role_in_namespace(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        namespace_services: web::Data<Arc<NamespaceService>>,
        role_rules: web::Data<Arc<RoleRules>>,
        namespace_id: web::Path<Uuid>,
        update_role: web::Json<UpdateUserRoleRequestDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = &config.secret_key;
        let user_id = extract_user_id_from_jwt_header(headers, secret_key)?;

        let update_role = update_role.into_inner();
        let namespace_id = namespace_id.into_inner();
        let new_role = update_role.role;
        let target_user = update_role.user_id;
        let role_rules_ref = role_rules.as_ref().as_ref();

        match namespace_services.check_user_namespace_perms(user_id, namespace_id, role_rules_ref, Permission::Update).await {
            Ok(res) => {
                if !res {
                    return Err(ServerError::RequestError(RequestError::PermissionDenied));
                }
            },
            Err(err) => return Err(err)
        }
        
        match namespace_services.compare_user_namespace_perms(user_id, target_user, namespace_id, role_rules_ref).await {
            Ok(res) => {
                if !res {
                    return Err(ServerError::RequestError(RequestError::PermissionDenied));
                }
            },
            Err(err) => return Err(err)
        }

        let new_role_enum = string_to_role(&new_role).ok_or_else(|| {
            ServerError::QueryError(QueryError::InvalidRole)
        })?;

        let new_role_weight = get_weight(&new_role_enum, &role_rules).unwrap();
        let user_role = match namespace_services.get_user_role_in_namespace(user_id, namespace_id).await {
            Ok(role) => role,
            Err(err) => return Err(err)
        };

        let user_role_enum = string_to_role(&user_role).ok_or_else(|| {
            ServerError::QueryError(QueryError::InvalidRole)
        })?;
        let user_role_weight = get_weight(&user_role_enum, &role_rules).unwrap();

        if user_role_weight < new_role_weight {
            return Err(ServerError::RequestError(RequestError::PermissionDenied));
        }

        match namespace_services.update_user_role_in_namespace(target_user, namespace_id, new_role).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }
}
