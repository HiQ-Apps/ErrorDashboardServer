use actix_web::web;

use crate::handlers::namespace_handlers::NamespaceHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/namespace")
            .wrap(jwt_middleware.clone())
            .route("/", web::post().to(NamespaceHandler::create_namespace))
            .route("/{id}", web::get().to(NamespaceHandler::get_namespace_by_id))
            .route("/user/{id}", web::get().to(NamespaceHandler::get_namespaces_by_user_id))
            .route("/{id}", web::put().to(NamespaceHandler::update_namespace))
            .route("/{id}", web::delete().to(NamespaceHandler::delete_namespace))
            .route("/{id}/errors", web::get().to(NamespaceHandler::get_errors_by_namespace_with_pagination))
            .route("/{id}/error/ws", web::get().to(NamespaceHandler::ws_index))
    );
}
