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
            .route("/{id}/invite", web::post().to(NamespaceHandler::invite_user_to_namespace))
            .route("/{namespace_id}/remove/{user_id}", web::delete().to(NamespaceHandler::remove_user_from_namespace))
            .route("/{id}/members", web::get().to(NamespaceHandler::get_namespace_members))
            .route("/{id}/user-role", web::get().to(NamespaceHandler::get_user_role_in_namespace))
            .route("/{id}/user-role", web::put().to(NamespaceHandler::update_user_role_in_namespace))
        );
}

pub fn configure_without_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ws/namespace")
            .route("/{id}/error", web::get().to(NamespaceHandler::namespace_error_ws_session))
                    );
}
