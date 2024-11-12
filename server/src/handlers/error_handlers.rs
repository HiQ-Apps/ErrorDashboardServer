use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::sync::Arc;
use uuid::Uuid;

use crate::{managers::namespace_manager::NamespaceServer, shared::utils::errors::RequestError};
use crate::services::error_services::ErrorService;
use crate::shared::utils::errors::ServerError;
use shared_types::{error_dtos::{CreateErrorRequest, UpdateErrorDTO}, extra_dtos::{ErrorPieChartQueryParams, ErrorMetadataQueryParams, TimeParams}};


pub struct ErrorHandler;

impl ErrorHandler {
    pub async fn create_error(
        req: HttpRequest,
        error_services: web::Data<Arc<ErrorService>>,
        namespace_manager: web::Data<Arc<NamespaceServer>>,
        new_error: web::Json<CreateErrorRequest>,
    ) -> Result<HttpResponse, ServerError> {
        println!("Creating Error");
        let error_dto = new_error.into_inner();
        let headers = req.headers();
        let client_id_header = headers.get("client_id").unwrap();

        let client_id = match client_id_header.to_str() {
            Ok(client_id) => client_id,
            Err(_) => return Err(ServerError::RequestError(RequestError::InvalidHeader))
        };
        let client_id = Uuid::parse_str(client_id).unwrap();
        
        let result = error_services.create_error(error_dto.clone(), client_id).await;

        match result {
            Ok(error) => {
                namespace_manager.broadcast_error(error).await;
                Ok(HttpResponse::Ok().finish())
            },
            Err(err) => Err(err),
        }
    }

    pub async fn get_error_by_id(
        error_services: web::Data<Arc<ErrorService>>,
        error_id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match error_services.get_error_by_id(error_id.into_inner()).await {
            Ok(error_dto) => Ok(HttpResponse::Ok().json(error_dto)),
            Err(err) => Err(err)
        }
    }

    pub async fn update_error(
        error_services: web::Data<Arc<ErrorService>>,
        update_error: web::Json<UpdateErrorDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let error_dto = update_error.into_inner();
     
        match error_services.update_error(error_dto).await {
            Ok(error_dto) => Ok(HttpResponse::Ok().json(error_dto)),
            Err(err) => Err(err)
        }
    }


    pub async fn get_aggregate_errors_by_date(
        error_services: web::Data<Arc<ErrorService>>,
        namespace_id: web::Path<Uuid>,
        time_params_query: web::Query<TimeParams>,
    ) -> Result<HttpResponse, ServerError> {
        let time_params = time_params_query.into_inner();
        let namespace_id = namespace_id.into_inner();
        let time_interval_minutes = time_params.time_interval_minutes;
        let start_time = time_params.start_time;
        let timezone = time_params.timezone;
        match error_services.get_aggregate_errors_by_date(namespace_id, start_time, time_interval_minutes, timezone).await {
            Ok(errors) => Ok(HttpResponse::Ok().json(errors)),
            Err(err) => Err(err)
        }
    }

    pub async fn get_error_metadata_by_group(
        namespace_services: web::Data<Arc<ErrorService>>,
        namespace_id: web::Path<Uuid>,
        query_params: web::Query<ErrorMetadataQueryParams>,
    ) -> Result<HttpResponse, ServerError> {
        let result = namespace_services.get_error_metadata_by_group(
            *namespace_id,
            query_params.group_by.clone(),
            query_params.group_key.clone(),
            query_params.offset as usize,
            query_params.limit as usize,
        ).await.map_err(|err| ServerError::from(err))?;

        Ok(HttpResponse::Ok().json(result))  
    }

    pub async fn get_error_metrics_pie_chart(
        namespace_services: web::Data<Arc<ErrorService>>,
        namespace_id: web::Path<Uuid>,
        query_params: web::Query<ErrorPieChartQueryParams>, 
    ) -> Result<HttpResponse, ServerError> {
        let result = namespace_services.get_error_metrics_pie_chart(
            *namespace_id,
            query_params.group_by.clone(),
        ).await.map_err(|err| ServerError::from(err))?;

        Ok(HttpResponse::Ok().json(result))
    }

    pub async fn get_unique_error_meta_by_namespace(
        namespace_services: web::Data<Arc<ErrorService>>,
        namespace_id: web::Path<Uuid>,
        filter_request: web::Query<String>,
    ) -> Result<HttpResponse, ServerError> {
        let filter = filter_request.into_inner();
        let namespace_id = namespace_id.into_inner();

        let result = namespace_services.get_unique_error_meta_by_namespace(namespace_id, filter).await.map_err(|err| ServerError::from(err))?;

        Ok(HttpResponse::Ok().json(result))
    }
}
