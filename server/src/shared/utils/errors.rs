use actix_web::{Error as ActixError, HttpResponse, http::StatusCode, error::ResponseError};
use anyhow::Error as AnyhowError;
use bcrypt::BcryptError;
use jsonwebtoken::errors::Error as JwtError;
use sea_orm::error::{DbErr, SqlErr};
use sea_orm::TransactionError;
use serde_json::Error as JsonError;
use thiserror::Error;
use uuid::Error as UuidError;

// Group enum'd errors into a single enum
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("External error: {0}")]
    ExternalError(ExternalError),

    #[error("Query error: {0}")]
    QueryError(QueryError),

    #[error("Request error: {0}")]
    RequestError(RequestError),

    #[error("Service initialization error: {0}")]
    ServiceInitError(String),

    #[error("HTTP error: {0} - {1}")]
    HttpError(StatusCode, String),

}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::QueryError(ref err) => {
                let status = match err {
                    QueryError::UserNotFound | QueryError::NamespaceNotFound | QueryError::UserNamespaceJunctionNotFound => StatusCode::NOT_FOUND,
                    QueryError::UserExists | QueryError::NamespaceExists | QueryError::UserNamespaceJunctionExists => StatusCode::CONFLICT,
                    _ => StatusCode::BAD_REQUEST,
                };
                HttpResponse::build(status).json(format!("{}", self))
            },
            ServerError::RequestError(_) => HttpResponse::build(StatusCode::BAD_REQUEST).json(format!("{}", self)),
            ServerError::HttpError(status, message) => HttpResponse::build(*status).json(message),
            _ => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(format!("{}", self)),
        }
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

    #[error("Transaction error: {0}")]
    Transaction(TransactionError<DbErr>),
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

// External Error implementations
impl From<ActixError> for ExternalError {
    fn from(error: ActixError) -> Self {
        ExternalError::Actix(error)
    }
}

impl From<AnyhowError> for ExternalError {
    fn from(error: AnyhowError) -> Self {
        ExternalError::Anyhow(error)
    }
}

impl From<DbErr> for ExternalError {
    fn from(error: DbErr) -> Self {
        ExternalError::DB(error)
    }
}

impl From<JwtError> for ExternalError {
    fn from(error: JwtError) -> Self {
        ExternalError::Jwt(error)
    }
}   

impl From<UuidError> for ExternalError {
    fn from(error: UuidError) -> Self {
        ExternalError::Uuid(error)
    }
}

impl From<BcryptError> for ExternalError {
    fn from(error: BcryptError) -> Self {
        ExternalError::Bcrypt(error)
    }
}

impl From<JsonError> for ExternalError {
    fn from(error: JsonError) -> Self {
        ExternalError::Json(error)
    }
}

impl From<SqlErr> for ExternalError {
    fn from(error: SqlErr) -> Self {
        ExternalError::Pool(error)
    }
}

impl From<TransactionError<DbErr>> for ServerError {
    fn from(error: TransactionError<DbErr>) -> Self {
        match error {
            TransactionError::Connection(err) => ServerError::ExternalError(ExternalError::DB(err)),
            TransactionError::Transaction(err) => ServerError::ExternalError(ExternalError::DB(err)),
        }
    }
}
