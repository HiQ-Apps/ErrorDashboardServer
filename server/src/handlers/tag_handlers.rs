use actix_web::{web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::tag_services::TagService;
use shared_types::tag_dtos::CreateTagDto;
use crate::shared::utils::errors::{ServerError, ExternalError};

pub struct TagHandler;

impl TagHandler {
    pub async fn create_tag(
        tag_services: web::Data<Arc<TagService>>,
        new_tag: web::Json<CreateTagDto>,
    ) -> Result<HttpResponse, ServerError> {
        let new_tag = new_tag.into_inner();
        match tag_services.create_tag(new_tag).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn delete_tag(
        tag_services: web::Data<Arc<TagService>>,
        id: web::Path<Uuid>,
    ) -> Result<HttpResponse, ServerError> {
        match tag_services.delete_tag(*id).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

}

