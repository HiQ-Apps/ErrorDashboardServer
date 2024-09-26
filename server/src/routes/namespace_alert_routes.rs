use std::{vec, sync::Arc};
use actix_web::web;
use actix_cors::Cors;

use crate::handlers::namespace_alert_handlers::NamespaceAlertHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/alert")
            .wrap(jwt_middleware.clone())
            .route("/", web::post().to(NamespaceAlertHandler::create_namespace_alert))
            .route("/{id}", web::put().to(NamespaceAlertHandler::update_namespace_alert))
            .route("/{id}", web::delete().to(NamespaceAlertHandler::delete_namespace_alert))
            .route("/namespace/{namespace_id}", web::get().to(NamespaceAlertHandler::get_namespace_alerts_by_namespace_id))
            .route("/user/{user_id}", web::get().to(NamespaceAlertHandler::get_namespace_alerts_by_user_id))
            .route("/subscribe", web::post().to(NamespaceAlertHandler::subscribe_user_to_namespace_alert))
            .route("/unsubscribe", web::post().to(NamespaceAlertHandler::unsubscribe_user_from_namespace_alert))
    );
}