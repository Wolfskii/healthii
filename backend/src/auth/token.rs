use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use jsonwebtoken::{decode, encode as encode_jwt, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn user_id(&self) -> Uuid {
        self.sub
    }

    pub fn session_id(&self) -> Uuid {
        self.jti
    }
}

pub fn encode(
    user_id: Uuid,
    session_id: Uuid,
    secret: &str,
    ttl: Duration,
) -> Result<String, AppError> {
    let now = now_secs();
    let claims = Claims {
        sub: user_id,
        jti: session_id,
        iat: now,
        exp: now + ttl.as_secs() as usize,
    };
    encode_jwt(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| AppError::internal(error.to_string()))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::unauthorized("Invalid or expired session"))
}

pub fn from_headers(headers: &HeaderMap, secret: &str) -> Result<Claims, AppError> {
    let token = bearer_token(headers)
        .or_else(|| cookie_token(headers))
        .ok_or_else(|| AppError::unauthorized("Authentication required"))?;
    decode_token(&token, secret)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim().to_string())
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookie.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("healthii_session=")
            .map(ToOwned::to_owned)
    })
}

fn now_secs() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as usize)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_token() {
        let user = Uuid::new_v4();
        let session = Uuid::new_v4();
        let secret = "test-jwt-secret-that-is-at-least-32-chars";
        let token = encode(user, session, secret, Duration::from_secs(60)).unwrap();
        let claims = decode_token(&token, secret).unwrap();
        assert_eq!(claims.user_id(), user);
        assert_eq!(claims.session_id(), session);
    }
}
