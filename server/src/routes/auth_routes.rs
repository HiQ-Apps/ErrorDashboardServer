use actix_web::web;

use crate::handlers::auth_handlers::AuthHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure_without_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(AuthHandler::login))
            .route("/register", web::post().to(AuthHandler::register))
            .route("/refresh", web::get().to(AuthHandler::refresh_access_token))
            .route("/login/google", web::get().to(AuthHandler::google_login))
            .route("/login/google/callback", web::get().to(AuthHandler::google_callback_handler))
    );
}

pub fn configure_with_auth(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("api/verified/auth")
            .wrap(jwt_middleware.clone())
            .route("/check", web::post().to(AuthHandler::verify_user))
    );
}

