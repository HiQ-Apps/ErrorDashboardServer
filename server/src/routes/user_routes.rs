use actix_web::web;

use crate::handlers::user_handlers::UserHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/users")
            // Unauthenticated routes
            .route("/{id}/verify", web::put().to(UserHandler::verify_user))
            .route("/{id}/{email}/reset-password", web::put().to(UserHandler::update_password))
            .route("/forgot-password", web::post().to(UserHandler::forgot_password))
            
            // Authenticated routes
            .service(
                web::resource("/{id}")
                    .wrap(jwt_middleware.clone())
                    .route(web::get().to(UserHandler::get_user)),
            )
            .service(
                web::resource("/{id}/profile")
                    .wrap(jwt_middleware.clone())
                    .route(web::get().to(UserHandler::get_user_profile))
                    .route(web::put().to(UserHandler::update_user_profile)),
            )
    );
}
