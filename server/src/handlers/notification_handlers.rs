use actix_web::{web, HttpRequest, HttpResponse};
use shared_types::extra_dtos::QueryParams;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::managers::notification_manager::NotificationServer;
use crate::services::notification_services::NotificationService;
use crate::shared::utils::errors::{ExternalError, ServerError};
use crate::shared::utils::jwt::extract_user_id_from_jwt_header;
use crate::handlers::ws_handlers::notification_ws_session;

pub struct NotificationHandler;

impl NotificationHandler {
    pub async fn get_notifications_by_user_id(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        notification_services: web::Data<Arc<NotificationService>>,
        query_params: web::Query<QueryParams>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;
        let query_params = query_params.into_inner();

        match notification_services.get_notifications_by_user_id(user_id, query_params).await {
            Ok(notifications) => Ok(HttpResponse::Ok().json(notifications)),
            Err(err) => Err(err)
        }
    }

    pub async fn seen_notification(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        notification_services: web::Data<Arc<NotificationService>>,
        notification_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;
        let notification_id = notification_id.into_inner();

        match notification_services.seen_notification(user_id, notification_id).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }

    pub async fn batch_seen_notifications(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        notification_services: web::Data<Arc<NotificationService>>,
        notification_ids_request: web::Json<Vec<Uuid>>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;
        let notification_ids = notification_ids_request.into_inner();

        match notification_services.batch_seen_notifications(user_id, notification_ids).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err)
        }
    }

    pub async fn notification_ws_session(
        req: HttpRequest,
        stream: web::Payload,
        user_id: web::Path<Uuid>,
        notification_server: web::Data<Arc<NotificationServer>>,
    ) -> Result<HttpResponse, ServerError> {
        let user_id = user_id.into_inner();
        let notification_server = notification_server.get_ref().clone();
        
        let (response, session, _msg_stream) = actix_ws::handle(&req, stream).map_err(|err| {
            ServerError::ExternalError(ExternalError::Actix(err))
        })?;

        actix_web::rt::spawn(notification_ws_session(
            session,
            user_id,
            notification_server
        ));

        Ok(response)
    }

    pub async fn clear_notifications(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        notification_services: web::Data<Arc<NotificationService>>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;
        

        
        Ok(HttpResponse::Ok().finish())
    }
}
