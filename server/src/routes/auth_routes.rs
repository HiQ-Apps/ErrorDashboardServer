use actix_web::web;

use crate::handlers::auth_handlers::AuthHandler;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(AuthHandler::login))
            .route("/register", web::post().to(AuthHandler::register))
            .route("/refresh", web::get().to(AuthHandler::refresh_access_token))
    );
}
