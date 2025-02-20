use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use futures::future::{ok, Ready};
use futures::Future;
use jsonwebtoken::{Algorithm, Validation};
use sea_orm::DatabaseConnection;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::config::Config;
use crate::shared::utils::errors::ServerError;
use crate::shared::utils::jwt::validate_jwt;

#[derive(Clone)]
pub struct JwtMiddleware {
    pub config: Arc<Config>,
    pub db_pool: Arc<DatabaseConnection>,
}
pub struct JwtTransform<S, E> {
    config: Arc<Config>,
    db_pool: Arc<DatabaseConnection>,
    service: S,
    phantom: PhantomData<E>,
}

impl<S, B, E> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = E> + 'static,
    S::Future: 'static,
    E: From<ServerError> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = E;
    type Transform = JwtTransform<S, E>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtTransform {
            config: self.config.clone(),
            db_pool: self.db_pool.clone(),
            service,
            phantom: PhantomData,
        })
    }
}

impl<S, B, E> Service<ServiceRequest> for JwtTransform<S, E>
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
        let headers = req.headers().clone();
        let cookies_result = req
            .cookies()
            .map(|cookies| cookies.iter().cloned().collect::<Vec<_>>());

        let mut found_token: Option<String> = None;

        if let Some(auth_header) = headers.get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                found_token = Some(auth_str.to_string());
            }
        }

        if found_token.is_none() {
            if let Ok(cookies) = cookies_result {
                if let Some(cookie) = cookies.iter().find(|c| c.name() == "access_token") {
                    found_token = Some(cookie.value().to_string());
                }
            }
        }

        let fut = self.service.call(req);

        let secret_key = self.config.secret_key.clone();
        let jwt_issuer = self.config.jwt_issuer.clone();
        let jwt_audience = self.config.jwt_audience.clone();
        let db_pool = self.db_pool.clone();

        Box::pin(async move {
            let res = fut.await?;

            let required_claims: Vec<&str> = vec!["exp"];
            let mut validation = Validation::new(Algorithm::HS256);
            validation.leeway = 60;
            validation.set_audience(&[&jwt_audience]);
            validation.set_issuer(&[jwt_issuer]);
            validation.set_required_spec_claims(&required_claims);
            validation.validate_exp = true;

            if let Some(token) = found_token {
                match validate_jwt(&token, &secret_key, &validation, &db_pool).await {
                    Ok(()) => Ok(res),
                    Err(err) => Err(E::from(err)),
                }
            } else {
                Err(E::from(ServerError::HttpError(
                    StatusCode::UNAUTHORIZED,
                    "No Authorization header or access_token cookie found.".to_string(),
                )))
            }
        })
    }
}
