use actix_web::web;

use crate::{handlers::admin_handlers::AdminHandler, middlewares::auth_middleware::JwtMiddleware};

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/admin")
            .wrap(jwt_middleware.clone())
            .route("/users", web::get().to(AdminHandler::get_all_user_data))
            .route(
                "/namespaces",
                web::get().to(AdminHandler::get_all_namespaces),
            )
            .route("/verify", web::post().to(AdminHandler::verify_admin)),
    );
}
