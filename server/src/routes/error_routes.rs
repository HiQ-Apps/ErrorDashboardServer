use actix_cors::Cors;
use actix_web::web;
use std::time::Duration;
use std::{sync::Arc, vec};

use crate::handlers::error_handlers::ErrorHandler;
use crate::middlewares::{
    auth_middleware::JwtMiddleware, rate_limit_middleware::RateLimiterMiddleware,
    sdk_auth_middleware::ClientAuthMiddleware,
};
use crate::shared::utils::rate_limit::DynamicStripedRateLimiter;

pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/error")
            .wrap(jwt_middleware.clone())
            .route("/{id}", web::get().to(ErrorHandler::get_error_by_id))
            .route("/", web::put().to(ErrorHandler::update_error))
            .route(
                "/aggregate/namespace/{namespace_id}",
                web::get().to(ErrorHandler::get_aggregate_errors_by_date),
            )
            .route(
                "/aggregate/details/namespace/{namespace_id}",
                web::get().to(ErrorHandler::get_error_metadata_by_group),
            )
            .route(
                "/aggregate/details/namespace/{namespace_id}/pie",
                web::get().to(ErrorHandler::get_error_metrics_pie_chart),
            )
            .route(
                "/unique/meta/namespace/{namespace_id}",
                web::get().to(ErrorHandler::get_unique_error_meta_by_namespace),
            ),
    );
}

pub fn sdk_configure(cfg: &mut web::ServiceConfig, client_sdk_middleware: &ClientAuthMiddleware) {
    let cors = Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["POST"])
        .allowed_headers(vec!["Content-Type", "client_id", "client_secret"])
        .max_age(3600);

    // Adjust as we scale
    let stripe =
        DynamicStripedRateLimiter::new(8, Duration::from_secs(60), 60, Duration::from_secs(1800));
    let rate_limiter_middleware = RateLimiterMiddleware::new(Arc::clone(&stripe));

    cfg.service(
        web::scope("/sdk/error")
            .wrap(client_sdk_middleware.clone())
            .wrap(rate_limiter_middleware)
            .wrap(cors)
            .route("/", web::post().to(ErrorHandler::create_error)),
    );
}
