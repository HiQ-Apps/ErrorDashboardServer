use actix_web::web;

use crate::handlers::user_handlers::UserHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;


pub fn configure_with_auth(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/users")
            .wrap(jwt_middleware.clone())
            .route("/{id}", web::get().to(UserHandler::get_user))
            .route("/{id}/profile", web::get().to(UserHandler::get_user_profile))
            .route("/{id}/profile", web::put().to(UserHandler::update_user_profile))
    );
}

pub fn configure_without_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/users")
        .route("/{id}/verify", web::put().to(UserHandler::verify_user))
        .route("/{id}/password", web::post().to(UserHandler::update_password))
        .route("/{id}/password", web::put().to(UserHandler::forgot_password))
       );
}
