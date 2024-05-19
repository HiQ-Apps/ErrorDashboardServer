use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::error::Error;

pub mod user_services;
pub use user_services::*;

pub mod auth_services;
pub use auth_services::*;

pub mod namespace_services;
pub use namespace_services::*;

pub mod error_services;
pub use error_services::*;

use crate::config::Config;
use crate::shared::utils::errors::ServerError;

pub fn init_services(db_pool: Arc<DatabaseConnection>, config: Arc<Config>) -> Result<(namespace_services::NamespaceService, user_services::UserService, auth_services::AuthService, error_services::ErrorService), Box<dyn Error>> {
    let namespace_service = namespace_services::NamespaceService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Namespace service failed to initialize".to_string()))?;

    let user_service = user_services::UserService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("User services failed to initialize".to_string()))?;

    let auth_service = auth_services::AuthService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Auth services failed to initialize".to_string()))?;

    let error_service = error_services::ErrorService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Error services failed to initialize".to_string()))?;


    Ok((namespace_service, user_service, auth_service, error_service))
}