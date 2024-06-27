use actix_web::web;

use crate::handlers::user_handlers::UserHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;


pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/users")
            .wrap(jwt_middleware.clone())
            .route("/{id}", web::get().to(UserHandler::get_user))
            .route("/{id}", web::delete().to(UserHandler::delete_user))
            .route("/{id}/profile", web::get().to(UserHandler::get_user_profile))
            .route("/{id}/profile", web::put().to(UserHandler::update_user_profile))
    );
}
