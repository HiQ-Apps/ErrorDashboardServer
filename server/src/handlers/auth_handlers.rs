use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, HttpRequest, Result};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use shared_types::user_dtos::{UserLoginDTO, UserCreateDTO, UserLoginResponseDTO};
use crate::config::Config;
use crate::services::AuthService;
use crate::shared::utils::errors::{ServerError, HttpError};


pub struct AuthHandler;

impl AuthHandler {
    pub async fn login(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
        login_data: web::Json<UserLoginDTO>
    ) -> Result<HttpResponse, ServerError> {
        let auth_services = AuthService::new(db.as_ref().clone(), config.as_ref().clone())?;
        let UserLoginDTO { email, password } = login_data.into_inner();

        match auth_services.login(email, password).await {
        Ok(user_service_response) => {
            let refresh_token = user_service_response.refresh_token.refresh_token.clone();
            let user_response = UserLoginResponseDTO::from(user_service_response);

            let cookie = Cookie::build("refresh_token", refresh_token)
                .http_only(true)
                .secure(false)
                .same_site(SameSite::Strict)
                .finish();
            Ok(HttpResponse::Ok().cookie(cookie).json(user_response))
        },
        Err(err) => Err(err),
        }
    }

    pub async fn register(
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>,
        new_user: web::Json<UserCreateDTO>
    ) -> Result<HttpResponse, ServerError> {
        let auth_services = AuthService::new(db.as_ref().clone(), config.as_ref().clone())?;
        
        match auth_services.register(new_user.username.clone(), new_user.email.clone(), new_user.password.clone()).await {
            Ok(id) => Ok(HttpResponse::Ok().json(id)),
            Err(ServerError::WebError(http_err)) => Ok(http_err.into()),
            Err(_) => Err(ServerError::WebError(HttpError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Internal server error".to_string(),
            })),
        }
    }

    pub async fn refresh_access_token(
        req: HttpRequest,
        db: web::Data<Arc<DatabaseConnection>>,
        config: web::Data<Arc<Config>>
    ) -> Result<HttpResponse, ServerError> {
        let auth_header = req.headers().get("refresh_token");

        match auth_header {
            Some(header) => {
                let header_str = header.to_str().map_err(|_| ServerError::InvalidHeader)?;
                let token = header_str
                    .strip_prefix("Bearer ")
                    .ok_or(ServerError::InvalidToken)?;

                let auth_services = AuthService::new(db.as_ref().clone(), config.as_ref().clone())?;
                let found_refresh = auth_services.find_by_token(token).await?;

                if let Some(token_model) = found_refresh {
                    let new_refresh_token = auth_services.process_token_refresh(&token_model.token).await?;
                    
                    let cookie = Cookie::build("refresh_token", new_refresh_token)
                        .http_only(true)
                        .secure(false)
                        .same_site(SameSite::Strict)
                        .finish();

                    let access_token = auth_services.refresh_access_token(token.to_string()).await?;

                    Ok(HttpResponse::Ok().cookie(cookie).json(access_token))
                } else {
                    Err(ServerError::InvalidToken)
                }
            },
            None => Err(ServerError::MissingHeader)
        }
    }

}
