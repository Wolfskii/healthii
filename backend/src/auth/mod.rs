pub mod rate_limit;
pub mod token;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::User};

const MIN_PASSWORD_LENGTH: usize = 10;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: User,
}

#[derive(Debug, FromRow)]
struct UserRecord {
    id: Uuid,
    email: String,
    password_hash: String,
    display_name: String,
    timezone: String,
    locale: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        Self {
            id: value.id,
            email: value.email,
            display_name: value.display_name,
            timezone: value.timezone,
            locale: value.locale,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created", body = AuthResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already registered")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Response), AppError> {
    let email = normalize_email(&body.email)?;
    let display_name = body.display_name.trim();
    if display_name.is_empty() || display_name.len() > 120 {
        return Err(AppError::validation("Display name is required"));
    }
    validate_password(&body.password)?;

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&state.pool)
        .await?;
    if existing > 0 {
        return Err(AppError::conflict(
            "An account with this email already exists",
        ));
    }

    let password_hash = hash_password(&body.password)?;
    let record = sqlx::query_as::<_, UserRecord>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, display_name, timezone, locale, created_at, updated_at
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(display_name)
    .fetch_one(&state.pool)
    .await?;

    let response = issue_session(&state, record, None).await?;
    Ok((StatusCode::CREATED, response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authenticated", body = AuthResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let ip = client_ip(&headers);
    if !state.login_limiter.check(&ip) {
        return Err(AppError::RateLimited);
    }

    let email = normalize_email(&body.email)?;
    let record = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, display_name, timezone, locale, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await?;

    let Some(record) = record else {
        return Err(AppError::unauthorized("Invalid email or password"));
    };

    if !verify_password(&body.password, &record.password_hash)? {
        return Err(AppError::unauthorized("Invalid email or password"));
    }

    issue_session(&state, record, headers.get(header::USER_AGENT)).await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Session revoked"))
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    if let Ok(claims) = token::from_headers(&headers, &state.config.jwt_secret) {
        sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
            .bind(claims.session_id())
            .execute(&state.pool)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn issue_session(
    state: &AppState,
    record: UserRecord,
    user_agent: Option<&axum::http::HeaderValue>,
) -> Result<Response, AppError> {
    let session_id = Uuid::new_v4();
    let token = token::encode(
        record.id,
        session_id,
        &state.config.jwt_secret,
        state.config.jwt_ttl,
    )?;
    let token_hash = hash_token(&token);
    let expires_at = Utc::now()
        + chrono::Duration::from_std(state.config.jwt_ttl)
            .unwrap_or_else(|_| chrono::Duration::hours(12));
    let agent = user_agent.and_then(|value| value.to_str().ok());

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, token_hash, user_agent, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(session_id)
    .bind(record.id)
    .bind(&token_hash)
    .bind(agent)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    let body = AuthResponse {
        token: token.clone(),
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_ttl.as_secs(),
        user: record.into(),
    };

    let mut response = Json(body).into_response();
    let cookie = format!(
        "healthii_session={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age={}",
        state.config.jwt_ttl.as_secs()
    );
    if let Ok(value) = cookie.parse() {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    Ok(response)
}

pub fn normalize_email(email: &str) -> Result<String, AppError> {
    let email = email.trim().to_ascii_lowercase();
    if email.len() < 5 || !email.contains('@') || email.contains(' ') {
        return Err(AppError::validation("A valid email address is required"));
    }
    Ok(email)
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AppError::validation(format!(
            "Password must be at least {MIN_PASSWORD_LENGTH} characters"
        )));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::internal(error.to_string()))
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed = PasswordHash::new(hash).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_email() {
        assert_eq!(
            normalize_email("  Ada@Healthii.app ").unwrap(),
            "ada@healthii.app"
        );
        assert!(normalize_email("nope").is_err());
    }

    #[test]
    fn rejects_short_passwords() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("long-enough-secret").is_ok());
    }
}
