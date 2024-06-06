
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{web, HttpResponse, HttpRequest, Result};
use shared_types::auth_dtos::VerifyUserDTO;
use std::sync::Arc;

use crate::config::Config;
use crate::shared::utils::jwt::{extract_user_id_from_jwt_cookie, extract_user_id_from_jwt_header};
use shared_types::user_dtos::{UserCreateDTO, UserLoginDTO, UserLoginServiceDTO, UserResponseDTO};
use crate::services::{AuthService, UserService};
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
    user_services: web::Data<Arc<UserService>>,
    config: web::Data<Arc<Config>>,
) -> Result<HttpResponse, ServerError> {
    let cookies = req.cookies().map_err(|_| ServerError::RequestError(RequestError::InvalidCookies))?;
    let refresh_token_cookie = cookies.iter().find(|cookie| cookie.name() == "refresh_token");

    match refresh_token_cookie {
        Some(refresh_token_cookie) => {
            let refresh_token = refresh_token_cookie.value();

            let token_model = auth_services.find_by_token(refresh_token).await?;
            match token_model {
                Some(token_model) => {
                    let refresh_token_response = auth_services.process_token_refresh(&token_model.token).await?;
                    let new_access_token = refresh_token_response.access_token.clone();

                    let new_access_token_cookie = Cookie::build("access_token", new_access_token.clone())
                        .http_only(true)
                        .secure(false)
                        .same_site(SameSite::Strict)
                        .finish();

                    let new_refresh_token_cookie = Cookie::build("refresh_token", refresh_token_response.refresh_token.refresh_token.clone())
                        .http_only(true)
                        .secure(false)
                        .same_site(SameSite::Strict)
                        .finish();

                    let user_id = extract_user_id_from_jwt_cookie(&refresh_token_cookie, &config.secret_key)?;
                    let user = user_services.get_user(user_id).await?;

                    let user_response = UserResponseDTO {
                        user,
                        access_token: new_access_token,
                    };

                    Ok(HttpResponse::Ok()
                        .cookie(new_access_token_cookie)
                        .cookie(new_refresh_token_cookie)
                        .json(user_response))
                },
                None => Err(ServerError::RequestError(RequestError::InvalidToken)),
            }
        },
        None => Err(ServerError::RequestError(RequestError::MissingCookie)),
    }
}

    pub async fn verify_user(
        req: HttpRequest,
        config: web::Data<Arc<Config>>,
        auth_services: web::Data<Arc<AuthService>>,
        password: web::Json<VerifyUserDTO>,
    ) -> Result<HttpResponse, ServerError> {
        let headers = req.headers();
        let secret_key = config.secret_key.clone();
        let user_id = extract_user_id_from_jwt_header(headers, &secret_key)?;

        let VerifyUserDTO { password } = password.into_inner();

        match auth_services.verify_user(user_id, password).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(err),
        }
    }
}
