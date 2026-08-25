use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::AuthUser};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Appointment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub location: Option<String>,
    pub provider: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertAppointment {
    pub title: String,
    pub location: Option<String>,
    pub provider: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Appointment>>, AppError> {
    let rows = sqlx::query_as::<_, Appointment>(
        r#"
        SELECT id, user_id, title, location, provider, starts_at, ends_at, notes, created_at
        FROM appointments
        WHERE user_id = $1
        ORDER BY starts_at ASC
        LIMIT 200
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertAppointment>,
) -> Result<(StatusCode, Json<Appointment>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(insert(&state, user.id, &body).await?),
    ))
}

pub async fn insert(
    state: &AppState,
    user_id: Uuid,
    body: &UpsertAppointment,
) -> Result<Appointment, AppError> {
    let title = body.title.trim();
    if title.is_empty() {
        return Err(AppError::validation("A title is required"));
    }
    sqlx::query_as::<_, Appointment>(
        r#"
        INSERT INTO appointments (user_id, title, location, provider, starts_at, ends_at, notes)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        RETURNING id, user_id, title, location, provider, starts_at, ends_at, notes, created_at
        "#,
    )
    .bind(user_id)
    .bind(title.chars().take(160).collect::<String>())
    .bind(trim_opt(body.location.as_deref(), 160))
    .bind(trim_opt(body.provider.as_deref(), 160))
    .bind(body.starts_at)
    .bind(body.ends_at)
    .bind(trim_opt(body.notes.as_deref(), 2000))
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM appointments WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Appointment not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn upcoming(
    state: &AppState,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<Appointment>, AppError> {
    sqlx::query_as::<_, Appointment>(
        r#"
        SELECT id, user_id, title, location, provider, starts_at, ends_at, notes, created_at
        FROM appointments
        WHERE user_id = $1 AND starts_at >= now()
        ORDER BY starts_at ASC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
}

fn trim_opt(value: Option<&str>, max: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max).collect())
}
