use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;

use crate::services::feature_request_services::FeatureRequestService;
use crate::shared::utils::errors::ServerError;
use shared_types::feature_request_dtos::{CreateFeatureRequestDTO, UpdateFeatureRequestStatusDTO};

pub struct FeatureRequestHandler;

impl FeatureRequestHandler {
    pub async fn create_feature_request(
        feature_request_services: web::Data<Arc<FeatureRequestService>>,
        create_feature_request: web::Json<CreateFeatureRequestDTO>,
    ) -> Result<(HttpResponse), ServerError> {
        let create_feature_request = create_feature_request.into_inner();
        match feature_request_services
            .create_feature_request(create_feature_request)
            .await
        {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn get_all_feature_requests(
        feature_request_services: web::Data<Arc<FeatureRequestService>>,
    ) -> Result<HttpResponse, ServerError> {
        match feature_request_services.get_all_feature_requests().await {
            Ok(feature_requests) => Ok(HttpResponse::Ok().json(feature_requests)),
            Err(err) => Err(err),
        }
    }

    pub async fn update_feature_request(
        feature_request_services: web::Data<Arc<FeatureRequestService>>,
        update_feature_request: web::Json<UpdateFeatureRequestStatusDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let update_feature_request = update_feature_request.into_inner();
        match feature_request_services
            .update_feature_status(update_feature_request)
            .await
        {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn delete_feature_request(
        feature_request_services: web::Data<Arc<FeatureRequestService>>,
        id: web::Path<i32>,
    ) -> Result<HttpResponse, ServerError> {
        let id = id.into_inner();
        match feature_request_services.delete_feature_request(id).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }
}
