mod database;
mod config;
mod handlers;
mod middlewares;
mod models;
mod routes;
mod services;
mod shared {
    pub mod utils;
}

use actix_web::{middleware, web, App, HttpServer};
use log::{ error, info };
use std::sync::Arc;

use crate::middlewares::auth_middleware::JwtMiddleware;
use crate::routes::{auth_routes, namespace_routes, user_routes};
use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = match Config::from_env() {
        Ok(conf) => {
            env_logger::init();
            info!("Successfully loaded configurations.");
            Arc::new(conf)},
        Err(error) => {
            env_logger::init();
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


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&db_pool)))
            .app_data(web::Data::new(Arc::clone(&config)))
            .wrap(middleware::Logger::default())
            .configure(|cfg| user_routes::configure(cfg, &jwt_middleware))
            .configure(auth_routes::configure)
            .configure(namespace_routes::configure)
    })
    .bind(("127.0.0.1", config_for_server.api_port))?
    .run()
    .await
    
}

