use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform};
use futures::future::{ok, Ready};
use futures::Future;
use jsonwebtoken::{Validation, Algorithm};
use sea_orm::DatabaseConnection;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};

use crate::config::Config;
use crate::shared::utils::errors::ServerError;
use crate::shared::utils::jwt::validate_jwt;

#[derive(Clone)]
pub struct JwtMiddleware {
    pub config: Arc<Config>,
    pub db_pool: Arc<DatabaseConnection>,
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

pub struct JwtTransform<S, E> {
    config: Arc<Config>,
    db_pool: Arc<DatabaseConnection>,
    service: S,
    phantom: PhantomData<E>,
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

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers().clone();
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

            match validate_jwt(&headers, &secret_key, &validation, &db_pool).await {
                Ok(()) => Ok(res),
                Err(err) => Err(E::from(err)),
            }
        })
        }

}
