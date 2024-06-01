use actix_web::web;

use crate::handlers::error_handlers::ErrorHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/error")
            .wrap(jwt_middleware.clone())
            .route("/", web::post().to(ErrorHandler::create_error))
            .route("/{id}", web::get().to(ErrorHandler::get_error_by_id))
            .route("/", web::put().to(ErrorHandler::update_error))
    );
}
