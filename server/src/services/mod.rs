use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::error::Error;

pub mod user_services;
pub use user_services::*;

pub mod auth_services;
pub use auth_services::*;

pub mod namespace_services;
pub use namespace_services::*;

use crate::config::Config;
use crate::shared::utils::errors::ServerError;

pub fn init_services(db_pool: Arc<DatabaseConnection>, config: Arc<Config>) -> Result<(namespace_services::NamespaceService, user_services::UserService, auth_services::AuthService), Box<dyn Error>> {
    let namespace_service = namespace_services::NamespaceService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|e| Box::new(ServerError::ServiceInitError("namespace_service".to_string(), Box::new(e))) as Box<dyn Error>)?;

    let user_service = user_services::UserService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|e| Box::new(ServerError::ServiceInitError("user_service".to_string(), Box::new(e))) as Box<dyn Error>)?;

    let auth_service = auth_services::AuthService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|e| Box::new(ServerError::ServiceInitError("auth_service".to_string(), Box::new(e))) as Box<dyn Error>)?;

    Ok((namespace_service, user_service, auth_service))
}