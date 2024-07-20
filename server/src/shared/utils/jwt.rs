use actix_web::cookie::Cookie;
use actix_web::http::header::HeaderMap;
use bcrypt::verify;
use chrono::{Duration, Utc, DateTime};
use jsonwebtoken::{Header, Validation, TokenData, encode, decode, EncodingKey, DecodingKey};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use serde_json::to_value;
use uuid::Uuid;

use shared_types::auth_dtos::{RefreshTokenDTO, Claims};

use crate::config::Config;
use crate::models::user_model::{Entity as UserEntity, Model as UserModel};
use crate::models::namespace_model::Entity as NamespaceEntity;
use crate::models::refresh_token_model::Model as RefreshTokenModel;
use crate::shared::utils::errors::{ServerError, QueryError, ExternalError, RequestError};

pub async fn validate_jwt(token: &str, secret_key: &str, validation: &Validation, db: &DatabaseConnection) -> Result<(), ServerError> {
    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());

    let token_data: TokenData<Claims> = decode(token, &decoding_key, validation).map_err(|err| ServerError::from(ExternalError::Jwt(err)))?;

    let uid = token_data.claims.sub;

    let found_user = UserEntity::find_by_id(uid)
        .one(db).await
        .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

    match found_user {
        Some(_user) => Ok(()),
        None => Err(ServerError::from(QueryError::UserNotFound)),
    }
}

pub async fn validate_namespace_secret_jwt(client_id: Uuid, client_secret: String, db: &DatabaseConnection) -> Result<bool, ServerError> {
    let found_namespace = NamespaceEntity::find()
        .filter(<NamespaceEntity as EntityTrait>::Column::ClientId.eq(client_id))
        .one(db)
        .await
        .map_err(|err| ServerError::from(ExternalError::DB(err)))?;

    match found_namespace {
        Some(namespace) => {
            verify(client_secret, &namespace.client_secret.to_string())
                .map_err(|err| ServerError::from(ExternalError::Bcrypt(err)));
            Ok(true)
        },
        None => Err(ServerError::from(QueryError::NamespaceNotFound)),
    }
}

pub fn extract_user_id_from_jwt_header(headers: &HeaderMap, secret_key: &str) -> Result<Uuid, ServerError> {
    if let Some(token_header) = headers.get("Authorization") {
        let token_str = token_header.to_str().unwrap_or("");

        let decoding_key = DecodingKey::from_secret(secret_key.as_ref());

        let token_data: TokenData<Claims> = decode(token_str, &decoding_key, &Validation::default())
            .map_err(|err| ServerError::ExternalError(ExternalError::Jwt(err)))?;

        Ok(token_data.claims.sub)
    } else {
        Err(ServerError::from(RequestError::InvalidToken))
    }
}

pub fn extract_user_id_from_jwt_cookie(cookies: &Cookie, secret_key: &str) -> Result<Uuid, ServerError> {
    let token_str = cookies.value();

    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());

    let token_data: TokenData<Claims> = decode(token_str, &decoding_key, &Validation::default())
        .map_err(|err| ServerError::from(ExternalError::Jwt(err)))?;

    Ok(token_data.claims.sub)
}

pub fn create_access_token(user: UserModel, configs: &Config) -> Result<String, ServerError> {
    let secret_key = &configs.secret_key;
    let jwt_iss = configs.jwt_issuer.clone();
    let jwt_aud = configs.jwt_audience.clone();

    let user_id = user.id;
    let user_data = match to_value(&user) {
        Ok(json) => Some(json),
        Err(err) => return Err(ServerError::from(ExternalError::Json(err)))
    };

    let now: DateTime<Utc> = Utc::now();
    let expiry: DateTime<Utc> = Utc::now() + Duration::hours(1);

    let claims = Claims {
        sub:user_id,
        iat:now,
        exp:expiry,
        iss:jwt_iss,
        aud:jwt_aud,
        data:user_data,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key.as_bytes()));
    match token {
        Ok(token) => Ok(token),
        Err(err) => return Err(ServerError::from(ExternalError::Jwt(err))),
    }
}

pub fn create_refresh_token(user_id: Uuid, configs: &Config) -> Result<RefreshTokenDTO, ServerError> {
    let jwt_iss = configs.jwt_issuer.clone();
    let jwt_aud = configs.jwt_audience.clone();
    let secret_key = &configs.secret_key;
    let now: DateTime<Utc> = Utc::now();
    let expiry: DateTime<Utc> = now + Duration::hours(12);

    let claims = Claims {
        sub: user_id,
        iat: now,
        exp: expiry,
        iss: jwt_iss.clone(),
        aud: jwt_aud.clone(),
        data: None,
    };
    
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key.as_bytes()))
        .map_err(|err| ServerError::from(ExternalError::Jwt(err)))?;
    
    let refresh_token_dto = RefreshTokenDTO {
        refresh_token: token,
        issued_at: now,
        expires_at: expiry,
        jwt_iss,
        jwt_aud,
        revoked: false
    };

    Ok(refresh_token_dto)
}


pub async fn refresh_access_token_util(refresh_token: RefreshTokenModel, db: &DatabaseConnection, configs: &Config) -> Result<String, ServerError> {
    let secret_key = configs.secret_key.as_bytes();
  
    let decoded_token = decode::<Claims>(
        &refresh_token.token,&DecodingKey::from_secret(secret_key),&Validation::default(),)
        .map_err(|err| ServerError::from(ExternalError::Jwt(err)))?;

    let uuid = decoded_token.claims.sub;
    
    let user = UserEntity::find_by_id(uuid)
        .one(db)
        .await
        .map_err(|err| ServerError::from(ExternalError::DB(err)))?;
    
    let user = match user {
        Some(user) => user.clone(),
        None => return Err(ServerError::from(QueryError::UserNotFound)),
    };

    let access_token = create_access_token(user, configs)?;

    Ok(access_token)
}
