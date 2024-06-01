
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{web, HttpResponse, HttpRequest, Result};
use chrono::Utc;
use std::sync::Arc;

use shared_types::user_dtos::{UserCreateDTO, UserLoginDTO, UserLoginServiceDTO, UserResponseDTO};
use crate::services::AuthService;
use crate::shared::utils::errors::{ServerError, RequestError};

pub struct AuthHandler;

impl AuthHandler {
    pub async fn login(
        auth_services: web::Data<Arc<AuthService>>,
        login_data: web::Json<UserLoginDTO>
    ) -> Result<HttpResponse, ServerError> {
        let UserLoginDTO { email, password } = login_data.into_inner();

        match auth_services.login(email, password).await {
            Ok(user_service_response) => {
                let UserLoginServiceDTO { user, access_token, refresh_token } = user_service_response;
                let refresh_token_value = refresh_token.refresh_token.clone();

                let user_response = UserResponseDTO {
                    user,
                    access_token: access_token.clone(),
                };

                let refresh_token_cookie = Cookie::build("refresh_token", refresh_token_value)
                    .http_only(true)
                    .path("/")
                    .secure(false)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(1))
                    .finish();

                let access_token_cookie = Cookie::build("access_token", access_token)
                    .http_only(true)
                    .path("/")
                    .secure(false)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(1))
                    .finish();

                Ok(HttpResponse::Ok()
                    .cookie(refresh_token_cookie)
                    .cookie(access_token_cookie)
                    .json(user_response))
            },
            Err(err) => Err(err),
        }
    }

    pub async fn register(
        auth_services: web::Data<Arc<AuthService>>,
        new_user: web::Json<UserCreateDTO>
    ) -> Result<HttpResponse, ServerError> {
        let UserCreateDTO { username, email, password } = new_user.into_inner();

        match auth_services.register(username, email, password).await {
            Ok(user_service_response) => {
                let UserLoginServiceDTO { user, access_token, refresh_token } = user_service_response;
                let refresh_token_value = refresh_token.refresh_token.clone();

                let user_response = UserResponseDTO {
                    user,
                    access_token: access_token.clone(),
                };

                let refresh_token_cookie = Cookie::build("refresh_token", refresh_token_value)
                    .http_only(true)
                    .secure(false)
                    .same_site(SameSite::Strict)
                    .finish();

                let access_token_cookie = Cookie::build("access_token", access_token)
                    .http_only(true)
                    .secure(false)
                    .same_site(SameSite::Strict)
                    .finish();

                Ok(HttpResponse::Ok()
                    .cookie(refresh_token_cookie)
                    .cookie(access_token_cookie)
                    .json(user_response))
            },
            Err(err) => Err(err), 
        }
    }

    pub async fn refresh_access_token(
        req: HttpRequest,
        auth_services: web::Data<Arc<AuthService>>,
    ) -> Result<HttpResponse, ServerError> {
        let auth_header = req.headers().get("refresh_token");

        match auth_header {
            Some(header) => {
                let header_str = header.to_str().map_err(|_| ServerError::RequestError(RequestError::InvalidHeader))?;
                let token = header_str
                    .strip_prefix("Bearer ")
                    .ok_or(ServerError::RequestError(RequestError::InvalidToken))?;

                match auth_services.find_by_token(token).await {
                    Ok(Some(token_model)) => {
                        let refresh_token_response = auth_services.process_token_refresh(&token_model.token).await?;
                        
                        let refresh_token_dto = refresh_token_response.refresh_token.clone();

                        let cookie = Cookie::build("refresh_token", refresh_token_dto.refresh_token)
                            .http_only(true)
                            .secure(false)
                            .same_site(SameSite::Strict)
                            .finish();

                        Ok(HttpResponse::Ok().cookie(cookie).json(refresh_token_response))
                    },
                    Ok(None) => Err(ServerError::RequestError(RequestError::InvalidToken)),
                    Err(e) => Err(e),
                }
            },
            None => Err(ServerError::RequestError(RequestError::MissingHeader))
        }
    }
}
