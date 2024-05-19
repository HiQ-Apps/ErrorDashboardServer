use actix_web::web;

use crate::handlers::error_handlers::ErrorHandler;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/error")
            .route("/", web::post().to(ErrorHandler::create_error))
    );
}
