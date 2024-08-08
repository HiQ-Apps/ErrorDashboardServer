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

use actix::Actor;
use actix_web::{middleware, web};
use log::{ error, info };
use std::sync::Arc;
use shuttle_actix_web::{ShuttleActixWeb, ActixWebService};

use crate::middlewares::{auth_middleware::JwtMiddleware, sdk_auth_middleware::ClientAuthMiddleware};
use crate::routes::{auth_routes, error_routes, namespace_routes, user_routes, tag_routes, static_routes};
use crate::services::init_services;
use crate::managers::namespace_manager::NamespaceServer;
use config::Config;


#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    let config = match Config::from_env() {
        Ok(conf) => {
            info!("Successfully loaded configurations.");
            Arc::new(conf)},
        Err(error) => {
            error!("Failed to load configurations: {}", error);
            std::process::exit(1)
        }
    };

    let config_for_bind = Arc::clone(&config);
    let config_for_server = Arc::clone(&config_for_bind);

    let db_pool = match database::create_pool(Arc::clone(&config)).await {
        Ok(pool) => {
            info!("Successfully connected to database.");
            Arc::new(pool)
        },
        Err(error) => {
            error!("Failed to create database pool: {}", error);
            std::process::exit(1);
        },
    };

    let jwt_middleware = JwtMiddleware {
        config: Arc::clone(&config_for_bind),
        db_pool: Arc::clone(&db_pool),
    };

    let sdk_middleware = ClientAuthMiddleware {
        db_pool: Arc::clone(&db_pool),
    };

    println!(r#"
         _____         _  _    _         _  _       _                                                             
        |_   _|       (_)| |  (_)       | |(_)     (_)                                                            
          | |   _ __   _ | |_  _   __ _ | | _  ____ _  _ __    __ _   ___   ___  _ __ __   __ ___  _ __           
          | |  | '_ \ | || __|| | / _` || || ||_  /| || '_ \  / _` | / __| / _ \| '__|\ \ / // _ \| '__|          
         _| |_ | | | || || |_ | || (_| || || | / / | || | | || (_| | \__ \|  __/| |    \ V /|  __/| |     _  _  _ 
        |_____||_| |_||_| \__||_| \__,_||_||_|/___||_||_| |_| \__, | |___/ \___||_|     \_/  \___||_|    (_)(_)(_)
                                                               __/ |                                              
                                                              |___/                                               
            "#);
    println!("Listening on {}:{}...", config_for_server.db_host, config_for_server.api_port);

    let (namespace_service, user_service, auth_service, error_service, tag_service ) = match init_services(db_pool.clone(), config.clone()) {
        Ok(services) => services,
        Err(e) => {
            error!("Failed to initialize services: {}", e);
            std::process::exit(1);
        }
    };

    let namespace_service = Arc::new(namespace_service);
    let user_service = Arc::new(user_service);
    let auth_service = Arc::new(auth_service);
    let error_service = Arc::new(error_service);
    let tag_service = Arc::new(tag_service);

    let namespace_manager = NamespaceServer::new().start();

    let configure_services = move |cfg: &mut web::ServiceConfig| {
            // DB Pool and Configs
            cfg.app_data(web::Data::new(Arc::clone(&db_pool)))
            .app_data(web::Data::new(Arc::clone(&config)))
            
            // Pass service as app_data to handlers and routes to make accessable
            .app_data(web::Data::new(namespace_service.clone()))
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(error_service.clone()))
            .app_data(web::Data::new(tag_service.clone()))
            
            // Namespace websocket manager
            .app_data(web::Data::new(namespace_manager.clone()))

            .service(
                web::scope("")
                    .wrap(middleware::Logger::default())
                    .configure(static_routes::configure)
                    .configure(auth_routes::configure_without_auth)
                    .configure(|cfg| auth_routes::configure_with_auth(cfg, &jwt_middleware))
                    .configure(|cfg| user_routes::configure(cfg, &jwt_middleware))
                    .configure(|cfg| namespace_routes::configure(cfg, &jwt_middleware))
                    .configure(|cfg| error_routes::configure(cfg, &jwt_middleware))
                    .configure(|cfg| error_routes::sdk_configure(cfg, &sdk_middleware))
                    .configure(|cfg| tag_routes::configure(cfg, &jwt_middleware))
            );
    };
    Ok(ActixWebService(configure_services))
}

