use std::fmt::{Formatter, Result, Display};
use std::error::Error;
use anyhow::Error as AnyhowError;
use sea_orm::error::{DbErr, SqlErr};
use serde_json::Error as JsonError;
use actix_web::{Error as ActixError, ResponseError as ActixResponseError, HttpResponse};
use actix_web::http::StatusCode;
use bcrypt::BcryptError;
use jsonwebtoken::errors::Error as JwtError;
use uuid::Error as UuidError;

#[derive(Debug)]
pub struct HttpError {
    pub status: StatusCode,
    pub message: String,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "HttpError: {} - {}", self.status, self.message)
    }
}

impl Error for HttpError {}

impl From<HttpError> for HttpResponse {
    fn from(err: HttpError) -> HttpResponse {
        HttpResponse::build(err.status).json(err.message)
    }
}

#[derive(Debug)]
pub enum ServerError {
    // 3rd party errors
    ActixError(ActixError),
    AnyhowError(AnyhowError),
    BcryptError(BcryptError),
    DBError(DbErr),
    JsonError(JsonError),
    JwtError(JwtError),
    PoolError(SqlErr),
    UuidError(UuidError),
    WebError(HttpError),

    // Query errors
    UserNotFound,
    NamespaceNotFound,

    // Request errors
    InvalidHeader,
    InvalidToken,
    MissingHeader,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            // 3rd party errors
            ServerError::ActixError(err) => write!(f, "ActixError: {}", err),
            ServerError::AnyhowError(err) => write!(f, "AnyhowError: {}", err),
            ServerError::BcryptError(err) => write!(f, "BcryptError: {}", err),
            ServerError::DBError(err) => write!(f, "DBError: {}", err),
            ServerError::JsonError(err) => write!(f, "JsonError: {}", err),
            ServerError::JwtError(err) => write!(f, "JwtError: {}", err),
            ServerError::PoolError(err) => write!(f, "PoolError: {}", err),
            ServerError::UuidError(err) => write!(f, "UuidError: {}", err),
            ServerError::WebError(err) => write!(f, "WebError: {}", err),

            // Query errors
            ServerError::UserNotFound => write!(f, "User not found"),
            ServerError::NamespaceNotFound => write!(f, "Namespace not found"),

            // Request errors
            ServerError::InvalidHeader => write!(f, "The provided header is invalid or not in the expected format"),
            ServerError::InvalidToken => write!(f, "The provided token is invalid"),
            ServerError::MissingHeader => write!(f, "The required header is missing from the request"),
        }
    }
}

impl Error for ServerError {}

impl ActixResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            // 3rd part error responses
            ServerError::WebError(http_err) => HttpResponse::build(http_err.status).json(http_err.message.clone()),
            ServerError::PoolError(_) | ServerError::DBError(_) | ServerError::AnyhowError(_) | ServerError::BcryptError(_) | ServerError::JsonError(_)
            | ServerError::UuidError(_) | ServerError::ActixError(_)
             => {HttpResponse::InternalServerError().json("Internal Server Error")},
            ServerError::JwtError(_) => HttpResponse::Unauthorized().json("Invalid JWT"),

            // Query error responses
            ServerError::UserNotFound => HttpResponse::Unauthorized().json("User not found"),
            ServerError::NamespaceNotFound => HttpResponse::NotFound().json("Namespace not found"),

            // Request error responses
            ServerError::MissingHeader => HttpResponse::BadRequest().json("Missing Authorization header"),
            ServerError::InvalidHeader => HttpResponse::BadRequest().json("Invalid Authorization header format"),
            ServerError::InvalidToken => HttpResponse::Unauthorized().json("Invalid Bearer token")
        }
    }
    
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::WebError(http_err) => http_err.status,
            ServerError::JwtError(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ActixError> for ServerError {
    fn from(err: ActixError) -> ServerError {
        ServerError::ActixError(err)
    }
}

impl From<AnyhowError> for ServerError {
    fn from(err: AnyhowError) -> ServerError {
        ServerError::AnyhowError(err)
    }
}

impl From<BcryptError> for ServerError {
    fn from(err: BcryptError) -> ServerError {
        ServerError::BcryptError(err)
    }
}

impl From<DbErr> for ServerError {
    fn from(err: DbErr) -> ServerError {
        ServerError::DBError(err)
    }
}

impl From<JsonError> for ServerError {
    fn from(err: JsonError) -> ServerError {
        ServerError::JsonError(err)
    }
}

impl From<JwtError> for ServerError {
    fn from(err: JwtError) -> ServerError {
        ServerError::JwtError(err)
    }
}

impl From<SqlErr> for ServerError {
    fn from(err: SqlErr) -> ServerError {
        ServerError::PoolError(err)
    }
}

impl From<UuidError> for ServerError {
    fn from(err: UuidError) -> ServerError {
        ServerError::UuidError(err)
    }
}

impl From<HttpError> for ServerError {
    fn from(err: HttpError) -> ServerError {
        ServerError::WebError(err)
    }
}
