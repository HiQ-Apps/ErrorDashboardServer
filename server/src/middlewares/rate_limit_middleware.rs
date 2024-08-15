use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform};
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};

use crate::shared::utils::errors::{ServerError, RequestError};
use crate::shared::utils::rate_limit::DynamicStripedRateLimiter;

pub struct RateLimiterMiddleware {
    rate_limiter: Arc<DynamicStripedRateLimiter>,
}

impl RateLimiterMiddleware {
    pub fn new(rate_limiter: Arc<DynamicStripedRateLimiter>) -> Self {
        RateLimiterMiddleware { rate_limiter }
    }
}

impl<S, B, E> Transform<S, ServiceRequest> for RateLimiterMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = E> + 'static,
    S::Future: 'static,
    B: 'static,
    E: From<ServerError> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = E;
    type InitError = ();
    type Transform = RateLimiterMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddlewareService {
            service: Arc::new(service),
            rate_limiter: self.rate_limiter.clone(),
        })
    }
}

/// Middleware service
pub struct RateLimiterMiddlewareService<S> {
    service: Arc<S>,
    rate_limiter: Arc<DynamicStripedRateLimiter>,
}

impl<S, B, E> Service<ServiceRequest> for RateLimiterMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = E> + 'static,
    S::Future: 'static,
    B: 'static,
    E: From<ServerError> + 'static,
{
    type Response = ServiceResponse<B>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Error = E;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Maybe not unknown
        let ip = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();

        let rate_limiter = self.rate_limiter.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            if rate_limiter.check_rate_limit(&ip) {
                fut.await
            } else {
                Err(E::from(ServerError::from(RequestError::RateLimitExceeded)))
            }
        })
    }
}
