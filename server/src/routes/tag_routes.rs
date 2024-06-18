use actix_web::web;

use crate::handlers::tag_handlers::TagHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/tag")
            .wrap(jwt_middleware.clone())
            .route("/", web::post().to(TagHandler::create_tag))
            .route("/{id}", web::delete().to(TagHandler::delete_tag))
    );
}

