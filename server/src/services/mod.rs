use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::error::Error;

pub mod user_services;
pub use user_services::*;

pub mod auth_services;
pub use auth_services::*;

pub mod feature_request_services;
pub use feature_request_services::*;

pub mod tag_services;
pub use tag_services::*;

pub mod namespace_alerts_services;
pub use namespace_alerts_services::*;

pub mod namespace_services;
pub use namespace_services::*;

pub mod error_services;
pub use error_services::*;

pub mod notification_services;
pub use notification_services::*;

use crate::config::Config;
use crate::shared::utils::errors::ServerError;

pub struct Services {
    pub auth_service: auth_services::AuthService,
    pub error_service: error_services::ErrorService,
    pub feature_request_service: feature_request_services::FeatureRequestService,
    pub namespace_service: namespace_services::NamespaceService,
    pub namespace_alerts_services: namespace_alerts_services::NamespaceAlertsService,
    pub notification_service: notification_services::NotificationService,
    pub tag_service: tag_services::TagService,
    pub user_service: user_services::UserService,
}

pub fn init_services(db_pool: Arc<DatabaseConnection>, config: Arc<Config>) -> Result<Services, Box<dyn Error>> {
    let namespace_service = namespace_services::NamespaceService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Namespace service failed to initialize".to_string()))?;

    let namespace_alerts_services = namespace_alerts_services::NamespaceAlertsService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Namespace alerts service failed to initialize".to_string()))?;
    
    let user_service = user_services::UserService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("User services failed to initialize".to_string()))?;

    let auth_service = auth_services::AuthService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Auth services failed to initialize".to_string()))?;

    let error_service = error_services::ErrorService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Error services failed to initialize".to_string()))?;

    let tag_service = tag_services::TagService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Tag services failed to initialize".to_string()))?;

    let notification_service = notification_services::NotificationService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Notification services failed to initialize".to_string()))?;

    let feature_request_service = feature_request_services::FeatureRequestService::new(Arc::clone(&db_pool), Arc::clone(&config))
        .map_err(|_| ServerError::ServiceInitError("Feature request services failed to initialize".to_string()))?;

    Ok(Services { namespace_service, namespace_alerts_services, user_service, auth_service, error_service, tag_service, notification_service, feature_request_service} )
}