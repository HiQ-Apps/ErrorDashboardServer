use actix_web::web;

use crate::handlers::feature_request_handlers::FeatureRequestHandler;
use crate::middlewares::auth_middleware::JwtMiddleware;


pub fn configure(cfg: &mut web::ServiceConfig, jwt_middleware: &JwtMiddleware) {
    cfg.service(
        web::scope("/api/feature_request")
            .wrap(jwt_middleware.clone())
            .route("/", web::post().to(FeatureRequestHandler::create_feature_request))
            .route("/", web::get().to(FeatureRequestHandler::get_all_feature_requests))
            .route("/{id}", web::get().to(FeatureRequestHandler::get_feature_request_by_id))
            .route("/", web::put().to(FeatureRequestHandler::update_feature_request))
            .route("/{id}", web::delete().to(FeatureRequestHandler::delete_feature_request))
    );
}

