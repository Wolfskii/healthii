use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{hash_password, hash_token, token, validate_password, verify_password},
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMe {
    pub display_name: Option<String>,
    pub timezone: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/me",
    tag = "users",
    security(("bearer_auth" = [])),
    request_body = UpdateMe,
    responses(
        (status = 200, body = User),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn update_me(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<UpdateMe>,
) -> Result<Json<User>, AppError> {
    let display_name = match body.display_name {
        Some(value) => Some(validate_display_name(&value)?),
        None => None,
    };
    let timezone = match body.timezone {
        Some(value) => Some(validate_timezone(&value)?),
        None => None,
    };
    if display_name.is_none() && timezone.is_none() {
        return Err(AppError::validation(
            "Provide a display name or timezone to update",
        ));
    }
    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET display_name = COALESCE($2, display_name),
            timezone = COALESCE($3, timezone),
            updated_at = now()
        WHERE id = $1
        RETURNING id, email, display_name, timezone, locale, created_at, updated_at
        "#,
    )
    .bind(user.id)
    .bind(display_name.as_deref())
    .bind(timezone.as_deref())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::unauthorized("Authentication required"))?;
    crate::audit::record(&state, user.id, "user.update", Some("user")).await;
    Ok(Json(updated))
}

pub(crate) fn validate_display_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() || name.len() > 120 {
        return Err(AppError::validation("Display name is required"));
    }
    Ok(name.to_string())
}

pub(crate) fn validate_timezone(tz: &str) -> Result<String, AppError> {
    let tz = tz.trim();
    if tz.eq_ignore_ascii_case("utc") {
        return Ok("UTC".into());
    }
    if tz.eq_ignore_ascii_case("gmt") {
        return Ok("GMT".into());
    }
    if tz.len() < 3 || tz.len() > 64 || !tz.contains('/') {
        return Err(AppError::validation(
            "Use an IANA timezone such as Europe/Warsaw, or UTC",
        ));
    }
    if tz.contains("//") || tz.starts_with('/') || tz.ends_with('/') {
        return Err(AppError::validation(
            "Use an IANA timezone such as Europe/Warsaw, or UTC",
        ));
    }
    if !tz
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '+' | '-'))
    {
        return Err(AppError::validation(
            "Use an IANA timezone such as Europe/Warsaw, or UTC",
        ));
    }
    Ok(tz.to_string())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
}

#[utoipa::path(
    put,
    path = "/api/v1/auth/password",
    tag = "auth",
    security(("bearer_auth" = [])),
    request_body = ChangePassword,
    responses((status = 204, description = "Password updated"))
)]
pub async fn change_password(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePassword>,
) -> Result<StatusCode, AppError> {
    validate_password(&body.new_password)?;
    let hash = sqlx::query_scalar::<_, String>("SELECT password_hash FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::unauthorized("Authentication required"))?;
    if !verify_password(&body.current_password, &hash)? {
        return Err(AppError::unauthorized("Current password is incorrect"));
    }
    let password_hash = hash_password(&body.new_password)?;
    sqlx::query("UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1")
        .bind(user.id)
        .bind(&password_hash)
        .execute(&state.pool)
        .await?;
    if let Some(current) = bearer_or_cookie(&headers) {
        sqlx::query(
            r#"
            UPDATE sessions
            SET revoked_at = now()
            WHERE user_id = $1 AND revoked_at IS NULL AND token_hash <> $2
            "#,
        )
        .bind(user.id)
        .bind(hash_token(&current))
        .execute(&state.pool)
        .await?;
    }
    crate::audit::record(&state, user.id, "auth.password_change", Some("user")).await;
    Ok(StatusCode::NO_CONTENT)
}

fn bearer_or_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            headers
                .get(header::COOKIE)
                .and_then(|value| value.to_str().ok())
                .and_then(|cookie| {
                    cookie.split(';').find_map(|part| {
                        part.trim()
                            .strip_prefix("healthii_session=")
                            .map(ToOwned::to_owned)
                    })
                })
        })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteAccount {
    pub password: String,
}

#[derive(Debug, FromRow)]
struct SessionListRow {
    id: Uuid,
    user_agent: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    token_hash: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionInfo {
    pub id: Uuid,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub current: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/sessions",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses((status = 200, body = [SessionInfo]))
)]
pub async fn list_sessions(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<SessionInfo>>, AppError> {
    let current_hash = bearer_or_cookie(&headers).map(|token| hash_token(&token));
    let rows = sqlx::query_as::<_, SessionListRow>(
        r#"
        SELECT id, user_agent, created_at, expires_at, token_hash
        FROM sessions
        WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > now()
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| SessionInfo {
                id: row.id,
                user_agent: row.user_agent,
                created_at: row.created_at,
                expires_at: row.expires_at,
                current: current_hash.as_ref() == Some(&row.token_hash),
            })
            .collect(),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/sessions/{id}",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses((status = 204))
)]
pub async fn revoke_session(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        "UPDATE sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(user.id)
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Session not found"));
    }
    crate::audit::record(&state, user.id, "auth.session_revoke", Some("session")).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/me",
    tag = "users",
    security(("bearer_auth" = [])),
    request_body = DeleteAccount,
    responses((status = 204))
)]
pub async fn delete_account(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<DeleteAccount>,
) -> Result<StatusCode, AppError> {
    let hash = sqlx::query_scalar::<_, String>("SELECT password_hash FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::unauthorized("Authentication required"))?;
    if !verify_password(&body.password, &hash)? {
        return Err(AppError::unauthorized("Password is incorrect"));
    }
    crate::documents::purge_for_user(&state, user.id).await?;
    crate::audit::purge_for_user(&state, user.id).await?;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_iana_timezones() {
        assert_eq!(validate_timezone("UTC").unwrap(), "UTC");
        assert_eq!(validate_timezone("utc").unwrap(), "UTC");
        assert_eq!(validate_timezone("Europe/Warsaw").unwrap(), "Europe/Warsaw");
        assert!(validate_timezone("nope").is_err());
        assert!(validate_timezone("/UTC").is_err());
        assert!(validate_timezone("Europe//Warsaw").is_err());
    }

    #[test]
    fn rejects_blank_display_names() {
        assert!(validate_display_name("  ").is_err());
        assert_eq!(validate_display_name(" Ada ").unwrap(), "Ada");
    }
}
