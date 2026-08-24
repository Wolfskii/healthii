use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{hash_token, token},
    error::AppError,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, ToSchema, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub timezone: String,
    pub locale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, FromRow)]
struct SessionRow {
    user_id: Uuid,
    email: String,
    revoked_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    tag = "users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user", body = User),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn me(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, display_name, timezone, locale, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::unauthorized("Authentication required"))?;

    Ok(axum::Json(user))
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = token::from_headers(&parts.headers, &state.config.jwt_secret)?;

        let authorization = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| {
                value
                    .strip_prefix("Bearer ")
                    .or_else(|| value.strip_prefix("bearer "))
            })
            .map(str::trim)
            .map(ToOwned::to_owned);

        let cookie_token = parts
            .headers
            .get(axum::http::header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|cookie| {
                cookie.split(';').find_map(|part| {
                    part.trim()
                        .strip_prefix("healthii_session=")
                        .map(ToOwned::to_owned)
                })
            });

        let raw_token = authorization
            .or(cookie_token)
            .ok_or_else(|| AppError::unauthorized("Authentication required"))?;

        let row = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT s.user_id, u.email, s.revoked_at, s.expires_at
            FROM sessions s
            INNER JOIN users u ON u.id = s.user_id
            WHERE s.id = $1 AND s.token_hash = $2
            "#,
        )
        .bind(claims.session_id())
        .bind(hash_token(&raw_token))
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid or expired session"))?;

        if row.revoked_at.is_some() || row.expires_at < Utc::now() {
            return Err(AppError::unauthorized("Invalid or expired session"));
        }

        if row.user_id != claims.user_id() {
            return Err(AppError::unauthorized("Invalid or expired session"));
        }

        Ok(Self {
            id: row.user_id,
            email: row.email,
        })
    }
}
