use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use futures::future::{ok, Ready};
use futures::Future;
use sea_orm::DatabaseConnection;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};
use uuid::Uuid;

use crate::shared::utils::errors::ServerError;
use crate::shared::utils::jwt::validate_namespace_secret_jwt;

#[derive(Clone)]
pub struct ClientAuthMiddleware {
    pub db_pool: Arc<DatabaseConnection>,
}

pub struct ClientAuthTransform<S, E> {
    db_pool: Arc<DatabaseConnection>,
    service: S,
    phantom: PhantomData<E>,
}

impl<S, B, E> Transform<S, ServiceRequest> for ClientAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = E> + 'static,
    S::Future: 'static,
    E: From<ServerError> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = E;
    type Transform = ClientAuthTransform<S, E>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ClientAuthTransform {
            db_pool: self.db_pool.clone(),
            service,
            phantom: PhantomData,
        })
    }
}

impl<S, B, E> Service<ServiceRequest> for ClientAuthTransform<S, E>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = E> + 'static,
    S::Future: 'static,
    E: From<ServerError> + 'static,
{
    type Response = ServiceResponse<B>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Error = E;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("client middleware called");
        let headers = req.headers().clone();

        let client_id = headers.get("client_id").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
        let client_secret = headers.get("client_secret").and_then(|h| h.to_str().ok()).map(|s| s.to_string());

        let fut = self.service.call(req);
        let db_pool = self.db_pool.clone();

        Box::pin(async move {
            if let (Some(id_str), Some(secret)) = (client_id, client_secret) {
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    match validate_namespace_secret_jwt(id, secret, &db_pool).await {
                        Ok(valid) if valid => {
                            let res = fut.await?;
                            Ok(res)
                        },
                        _ => Err(E::from(ServerError::HttpError(StatusCode::UNAUTHORIZED, "Invalid client_id or client_secret".to_string()))),
                    }
                } else {
                    Err(E::from(ServerError::HttpError(StatusCode::UNAUTHORIZED, "Invalid client_id format".to_string())))
                }
            } else {
                Err(E::from(ServerError::HttpError(StatusCode::UNAUTHORIZED, "Missing client_id or client_secret header".to_string())))
            }
        })
    }
}






