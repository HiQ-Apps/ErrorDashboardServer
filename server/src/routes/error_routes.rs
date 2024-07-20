use actix_web::web;

use crate::handlers::error_handlers::ErrorHandler;
use crate::middlewares::{auth_middleware::JwtMiddleware, sdk_auth_middleware::ClientAuthMiddleware};

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/error")
            .wrap(jwt_middleware.clone())
            .route("/{id}", web::get().to(ErrorHandler::get_error_by_id))
            .route("/", web::put().to(ErrorHandler::update_error))
            .route("/aggregate/namespace/{namespace_id}", web::get().to(ErrorHandler::get_aggregate_errors_by_date))
            .route("/aggregate/details/namespace/{namespace_id}", web::get().to(ErrorHandler::get_error_metadata_by_group))
    );
}

pub fn sdk_configure(cfg: &mut web::ServiceConfig, client_sdk_middleware: &ClientAuthMiddleware) {
    cfg.service(
        web::scope("/sdk/error")
            .wrap(client_sdk_middleware.clone())
            .route("/", web::post().to(ErrorHandler::create_error))
    );
}