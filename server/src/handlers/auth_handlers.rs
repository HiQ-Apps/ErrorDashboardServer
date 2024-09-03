
use actix_web::{web, HttpResponse, HttpRequest, Result, cookie::{Cookie, SameSite, time::Duration}};
use oauth2::{AuthorizationCode, TokenResponse, basic::BasicClient};
use reqwest::Client;
use std::sync::Arc;

use crate::config::Config;
use crate::services::{AuthService, UserService};
use crate::shared::utils::errors::{ServerError, RequestError};
use crate::shared::utils::jwt::{extract_user_id_from_jwt_cookie, extract_user_id_from_jwt_header};
use crate::libs::oauth_client::fetch_google_user_info;
use shared_types::auth_dtos::{VerifyUserDTO, CallbackQuery};
use shared_types::user_dtos::{UserCreateDTO, UserLoginDTO, UserLoginServiceDTO, UserResponseDTO, GoogleUserInfoDTO};

pub struct AuthHandler;

impl AuthHandler {
    pub async fn login(
        auth_services: web::Data<Arc<AuthService>>,
        login_data: web::Json<UserLoginDTO>
    ) -> Result<HttpResponse, ServerError> {
        let UserLoginDTO { email, password } = login_data.into_inner();

        match auth_services.login(email, password).await {
            Ok(user_service_response) => {
                let UserLoginServiceDTO { user, user_profile, access_token, refresh_token } = user_service_response;
                let refresh_token_value = refresh_token.refresh_token.clone();

                let user_response = UserResponseDTO {
                    user,
                    user_profile,
                    access_token: access_token.clone(),
                };

                let refresh_token_cookie = Cookie::build("refreshToken", refresh_token_value)
                    .http_only(true)
                    .path("/")
                    .secure(false)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(1))
                    .finish();

                let access_token_cookie = Cookie::build("accessToken", access_token)
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

    pub async fn google_login(auth_services: web::Data<Arc<AuthService>>, oauth_client: web::Data<BasicClient>) -> HttpResponse {
        let login = auth_services.google_login(oauth_client.get_ref().clone()).await;

        match login {
            Ok(login_url) => login_url,
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }

    pub async fn google_callback_handler(
        auth_service: web::Data<Arc<AuthService>>,
        oauth_client: web::Data<BasicClient>,
        query: web::Query<CallbackQuery>,
    ) -> HttpResponse {
        let token_result = oauth_client.get_ref()
            .exchange_code(AuthorizationCode::new(query.code.clone()))
            .request_async(oauth2::reqwest::async_http_client)
            .await;

        match token_result {
            Ok(token) => {
                let access_token = token.access_token().secret();

                match fetch_google_user_info(access_token).await {
                    Ok(user_info) => {
                        match auth_service.google_callback(user_info).await {
                            Ok(user_dto) => HttpResponse::Ok().json(user_dto),
                            Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
                        }
                    }
                    Err(_) => HttpResponse::InternalServerError().body("Failed to fetch user info"),
                }
            }
            Err(_) => HttpResponse::Unauthorized().body("OAuth Callback failed"),
        }
    }

    pub async fn register(
        auth_services: web::Data<Arc<AuthService>>,
        new_user: web::Json<UserCreateDTO>
    ) -> Result<HttpResponse, ServerError> {
        let UserCreateDTO { username, email, password } = new_user.into_inner();

        match auth_services.register(username, email, password).await {
            Ok(user_service_response) => {
                let UserLoginServiceDTO { user, user_profile, access_token, refresh_token } = user_service_response;
                let refresh_token_value = refresh_token.refresh_token.clone();

                let user_response = UserResponseDTO {
                    user,
                    user_profile,
                    access_token: access_token.clone(),
                };

                let refresh_token_cookie = Cookie::build("refreshToken", refresh_token_value)
                    .http_only(true)
                    .secure(false)
                    .path("/")
                    .same_site(SameSite::Strict)
                    .finish();

                let access_token_cookie = Cookie::build("accessToken", access_token)
                    .http_only(true)
                    .secure(false)
                    .path("/")
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
        let refresh_token_cookie = cookies.iter().find(|cookie| cookie.name() == "refreshToken");

        match refresh_token_cookie {
            Some(refresh_token_cookie) => {
                let refresh_token = refresh_token_cookie.value();

                let token_model = auth_services.find_by_token(refresh_token).await?;
                match token_model {
                    Some(token_model) => {
                        let refresh_token_response = auth_services.process_token_refresh(&token_model.token).await?;
                        let new_access_token = refresh_token_response.access_token.clone();

                        let new_access_token_cookie = Cookie::build("accessToken", new_access_token.clone())
                            .http_only(true)
                            .secure(false)
                            .path("/")
                            .same_site(SameSite::Strict)
                            .finish();

                        let new_refresh_token_cookie = Cookie::build("refreshToken", refresh_token_response.refresh_token.refresh_token.clone())
                            .http_only(true)
                            .secure(false)
                            .path("/")
                            .same_site(SameSite::Strict)
                            .finish();

                        let user_id = extract_user_id_from_jwt_cookie(&refresh_token_cookie, &config.secret_key)?;
                        let user = user_services.get_user(user_id).await?;
                        let user_profile = user_services.get_user_profile(user_id).await?;  

                        let user_response = UserResponseDTO {
                            user,
                            user_profile,
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
