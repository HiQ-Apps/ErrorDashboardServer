use actix_web::web;

use crate::handlers::namespace_handlers::NamespaceHandler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/namespace")
            .route("/", web::post().to(NamespaceHandler::create_namespace))
            .route("/{id}", web::get().to(NamespaceHandler::get_namespace_by_id))
            .route("/user/{id}", web::get().to(NamespaceHandler::get_namespaces_by_user_id))
            .route("/{id}", web::put().to(NamespaceHandler::update_namespace))
            .route("/{namespace_id}", web::delete().to(NamespaceHandler::delete_namespace))
            .route("/{namespace_id}/errors", web::get().to(NamespaceHandler::get_errors_by_namespace_with_pagination))
            .route("/{id}/error/ws", web::get().to(NamespaceHandler::ws_index))
    );
}
