mod database;
mod config;
mod handlers;
mod middlewares;
mod models;
mod routes;
mod services;
mod managers;
mod shared {
    pub mod utils;
}
mod libs;

use env_logger;
use actix_web::web;
use log::{error, info};
use managers::namespace_manager::NamespaceServer;
use managers::notification_manager::NotificationServer;
use std::sync::Arc;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;

use crate::middlewares::{auth_middleware::JwtMiddleware, sdk_auth_middleware::ClientAuthMiddleware};
use crate::routes::{admin_routes, auth_routes, bug_report_routes, error_routes, feature_request_routes, namespace_routes, namespace_alert_routes, user_routes, tag_routes, static_routes, ws_routes, notification_routes};
use crate::services::init_services;
use crate::shared::utils::role::initialize_role_rules;
use config::Config;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    env_logger::try_init().ok();

    let config = match Config::from_secret_store(secrets) {
        Ok(conf) => {
            info!("Successfully loaded configurations.");
            Arc::new(conf)
        }
        Err(error) => {
            error!("Failed to load configurations: {}", error);
            std::process::exit(1);
        }
    };

    let db_pool = match database::create_pool(Arc::clone(&config)).await {
        Ok(pool) => {
            info!("Successfully connected to database.");
            Arc::new(pool)
        }
        Err(error) => {
            error!("Failed to create database pool: {}", error);
            std::process::exit(1);
        }
    };

    let jwt_middleware = JwtMiddleware {
        config: Arc::clone(&config),
        db_pool: Arc::clone(&db_pool),
    };

    let sdk_middleware = ClientAuthMiddleware {
        db_pool: Arc::clone(&db_pool),
    };


    println!("Starting server...");

    let services =
        match init_services(db_pool.clone(), config.clone()) {
            Ok(services) => services,
            Err(e) => {
                error!("Failed to initialize services: {}", e);
                std::process::exit(1);
            }
        };

    let namespace_service = Arc::new(services.namespace_service);
    let namespace_alert_service = Arc::new(services.namespace_alerts_services);
    let user_service = Arc::new(services.user_service);
    let auth_service = Arc::new(services.auth_service);
    let bug_report_service = Arc::new(services.bug_report_service);
    let error_service = Arc::new(services.error_service);
    let tag_service = Arc::new(services.tag_service);
    let notification_service = Arc::new(services.notification_service);
    let feature_request_service = Arc::new(services.feature_request_service);
    let namespace_manager = Arc::new(NamespaceServer::new());
    let notification_manager = Arc::new(NotificationServer::new());
    
    let role_rules = Arc::new(initialize_role_rules());

    // Return a closure that configures the service
    let config = move |cfg: &mut web::ServiceConfig| {
        cfg.app_data(web::Data::new(Arc::clone(&db_pool)))
            .app_data(web::Data::new(Arc::clone(&config)))
            .app_data(web::Data::new(Arc::clone(&role_rules)))
            .app_data(web::Data::new(namespace_service.clone()))
            .app_data(web::Data::new(namespace_alert_service.clone()))
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(bug_report_service.clone()))
            .app_data(web::Data::new(error_service.clone()))
            .app_data(web::Data::new(tag_service.clone()))
            .app_data(web::Data::new(notification_service.clone()))
            .app_data(web::Data::new(feature_request_service.clone()))
            .app_data(web::Data::new(namespace_manager.clone()))
            .app_data(web::Data::new(notification_manager.clone()))
            .configure(static_routes::configure)
            .configure(auth_routes::configure_without_auth)
            .configure(ws_routes::configure_ws)
            .configure(|cfg| admin_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| auth_routes::configure_with_auth(cfg, &jwt_middleware))
            .configure(|cfg| bug_report_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| error_routes::sdk_configure(cfg, &sdk_middleware))
            .configure(|cfg| feature_request_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| user_routes::configure_user_routes(cfg, &jwt_middleware))
            .configure(|cfg| namespace_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| namespace_alert_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| notification_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| error_routes::configure(cfg, &jwt_middleware))
            .configure(|cfg| tag_routes::configure(cfg, &jwt_middleware));
    };

    Ok(config.into())
}
