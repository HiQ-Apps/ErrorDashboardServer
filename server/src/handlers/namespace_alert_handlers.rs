use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::shared::utils::errors::ServerError;
use shared_types::namespace_alert_dtos::{CreateNamespaceAlertRequestDTO, NamespaceAlertSubscriptionRequestDTO};
use crate::services::namespace_alerts_services::NamespaceAlertsService;

pub struct NamespaceAlertHandler;

impl NamespaceAlertHandler {
    pub async fn create_namespace_alert(
        namespace_alert_services:web::Data<Arc<NamespaceAlertsService>>,
        new_namespace_alert: web::Json<CreateNamespaceAlertRequestDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let new_namespace_alert = new_namespace_alert.into_inner();
        match namespace_alert_services.create_namespace_alert(new_namespace_alert).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn delete_namespace_alert(
        namespace_alert_services:web::Data<Arc<NamespaceAlertsService>>,
        namespace_alert_id: web::Path<Uuid>) -> Result<HttpResponse, ServerError> {
        let namespace_alert_id = namespace_alert_id.into_inner();
        match namespace_alert_services.delete_namespace_alert(namespace_alert_id).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn get_namespace_alerts_by_namespace_id(
        namespace_alert_services:web::Data<Arc<NamespaceAlertsService>>,
        namespace_id: web::Path<Uuid>) -> Result<HttpResponse, ServerError> {
        let namespace_id = namespace_id.into_inner();
        match namespace_alert_services.get_namespace_alerts_by_namespace_id(namespace_id).await {
            Ok(alerts) => Ok(HttpResponse::Ok().json(alerts)),
            Err(err) => Err(err),
        }
    }

    pub async fn get_namespace_alerts_by_user_id(
        namespace_alert_services:web::Data<Arc<NamespaceAlertsService>>,
        user_id: web::Path<Uuid>) -> Result<HttpResponse, ServerError> {
        let user_id = user_id.into_inner();
        match namespace_alert_services.get_namespace_alerts_by_user_id(user_id).await {
            Ok(alerts) => Ok(HttpResponse::Ok().json(alerts)),
            Err(err) => Err(err),
        }
    }

    pub async fn subscribe_user_to_namespace_alert(
        namespace_alert_services:web::Data<Arc<NamespaceAlertsService>>,
        subscription: web::Json<NamespaceAlertSubscriptionRequestDTO>
    ) -> Result<HttpResponse, ServerError> {
        let subscription = subscription.into_inner();
        match namespace_alert_services.subscribe_user_to_namespace_alert(subscription).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

    pub async fn unsubscribe_user_from_namespace_alert(
        namespace_alert_services:web::Data<Arc<NamespaceAlertsService>>,
        subscription: web::Json<NamespaceAlertSubscriptionRequestDTO>) -> Result<HttpResponse, ServerError> {
        let subscription = subscription.into_inner();
        match namespace_alert_services.unsubscribe_user_from_namespace_alert(subscription).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }

}

