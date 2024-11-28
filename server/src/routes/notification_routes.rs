use actix_web::web;

use crate::handlers::notification_handlers::NotificationHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/notification")
            .wrap(jwt_middleware.clone())
            .route("/", web::get().to(NotificationHandler::get_notifications_by_user_id))
            .route("/{notification_id}", web::put().to(NotificationHandler::seen_notification))
    );
}