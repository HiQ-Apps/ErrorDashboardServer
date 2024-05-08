use actix_web::{Error as ActixError, HttpResponse, http::StatusCode, error::HttpError, error::ResponseError};
use anyhow::Error as AnyhowError;
use bcrypt::BcryptError;
use jsonwebtoken::errors::Error as JwtError;
use sea_orm::error::{DbErr, SqlErr};
use serde_json::Error as JsonError;
use std::error::Error as StdError;
use thiserror::Error;
use uuid::Error as UuidError;

// Define External Errors grouped under ServerError
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("External error: {0}")]
    ExternalError(ExternalError),

    #[error("Query error: {0}")]
    QueryError(QueryError),

    #[error("Request error: {0}")]
    RequestError(RequestError),

    #[error("Service initialization error: {0} - {1:?}")]
    ServiceInitError(String, Box<dyn StdError>),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

// Implementation for actix ResponseError
impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        let status = match *self {
            ServerError::QueryError(ref err) => match err {
                QueryError::UserNotFound | QueryError::NamespaceNotFound | QueryError::UserNamespaceJunctionNotFound=> StatusCode::NOT_FOUND,
                QueryError::UserExists | QueryError::NamespaceExists | QueryError::UserNamespaceJunctionExists=> StatusCode::CONFLICT,
                _ => StatusCode::BAD_REQUEST,
            },
            ServerError::RequestError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status).json(format!("{}", self))
    }
}

#[derive(Debug, Error)]
pub enum ExternalError {
    #[error("Actix error: {0}")]
    Actix(ActixError),

    #[error("Anyhow error: {0}")]
    Anyhow(AnyhowError),

    #[error("Database error: {0}")]
    DB(DbErr),

    #[error("JWT error: {0}")]
    Jwt(JwtError),

    #[error("UUID error: {0}")]
    Uuid(UuidError),

    #[error("Bcrypt error: {0}")]
    Bcrypt(BcryptError),

    #[error("JSON error: {0}")]
    Json(JsonError),

    #[error("SQL pool error: {0}")]
    Pool(SqlErr),

    #[error("HTTP error: {0}")]
    Web(HttpError),
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserExists,

    #[error("Password incorrect")]
    PasswordIncorrect,

    #[error("Namespace not found")]
    NamespaceNotFound,

    #[error("Namespace already exists")]
    NamespaceExists,

    #[error("User-Namespace junction not found")]
    UserNamespaceJunctionNotFound,

    #[error("User-Namespace junction already exists")]
    UserNamespaceJunctionExists,
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Missing user ID")]
    MissingUserID,

    #[error("Invalid header")]
    InvalidHeader,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing header")]
    MissingHeader,

    #[error("Permission denied")]
    PermissionDenied,
}

impl From<ExternalError> for ServerError {
    fn from(err: ExternalError) -> Self {
        ServerError::ExternalError(err)
    }
}

impl From<QueryError> for ServerError {
    fn from(err: QueryError) -> Self {
        ServerError::QueryError(err)
    }
}

impl From<RequestError> for ServerError {
    fn from(err: RequestError) -> Self {
        ServerError::RequestError(err)
    }
}
