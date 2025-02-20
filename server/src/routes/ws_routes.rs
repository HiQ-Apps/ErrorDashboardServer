use actix_web::web;

use crate::handlers::namespace_handlers::NamespaceHandler;
use crate::handlers::notification_handlers::NotificationHandler;

pub fn configure_ws(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ws")
            .route(
                "/namespace/{id}/error",
                web::get().to(NamespaceHandler::namespace_error_ws_session),
            )
            .route(
                "/notifications/{id}",
                web::get().to(NotificationHandler::notification_ws_session),
            ),
    );
}
