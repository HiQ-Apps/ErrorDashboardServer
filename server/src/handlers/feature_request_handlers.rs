use actix_web::{web, HttpRequest, HttpResponse, Result};
use shared_types::extra_dtos::FilterRequest;
use std::sync::Arc;
use uuid::Uuid;

pub struct FeatureRequestHandler;

impl FeatureRequestHandler {
    pub async fn create_feature_request(        
    ) -> Result<HttpResponse, ServerError> {
        Ok(())
    }

    pub async fn get_all_feature_requests(
    ) -> Result<HttpResponse, ServerError> {
        Ok(())
    }

    pub async fn get_feature_request_by_id(
    ) -> Result<HttpResponse, ServerError> {
        Ok(())
    }

    pub async fn update_feature_request(
    ) -> Result<HttpResponse, ServerError> {
        Ok(())
    }

    pub async fn delete_feature_request(
    ) -> Result<HttpResponse, ServerError> {
        Ok(())
    }

}

