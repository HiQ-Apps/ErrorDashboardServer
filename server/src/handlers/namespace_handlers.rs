use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;


use shared_types::namespace_dtos::CreateNamespaceDto;
use crate::config::Config;
use crate::shared::utils::errors::{ServerError, HttpError};

pub struct NamespaceHandler;

impl NamespaceHandler {
    pub async fn create_namespace(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
        new_namespace: web::Json<CreateNamespaceDto>
    ) -> Result<HttpResponse, ServerError> {
    }

    pub async fn get_namespace_by_id(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
        namespace_id: web::Path<Uuid>
    ) -> Result<HttpResponse, ServerError> {
    }
    

    pub async fn get_namespaces_by_user_id(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
        user_id: web::Data<Arc<Uuid>>
    ) -> Result<HttpResponse, ServerError> {
    }

    
    pub async fn update_namespace(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
    ) -> Result<HttpResponse, ServerError> {
    }

    
    pub async fn delete_namespace(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
    ) -> Result<HttpResponse, ServerError> {
    }
}
