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
pub struct Symptom {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub severity: Option<i16>,
    pub noted_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSymptom {
    pub name: String,
    pub severity: Option<i16>,
    pub noted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

pub async fn recent(state: &AppState, user_id: Uuid, limit: i64) -> Result<Vec<Symptom>, AppError> {
    sqlx::query_as::<_, Symptom>(
        r#"
        SELECT id, user_id, name, severity, noted_at, notes, created_at
        FROM symptoms
        WHERE user_id = $1
        ORDER BY noted_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Symptom>>, AppError> {
    let rows = sqlx::query_as::<_, Symptom>(
        r#"
        SELECT id, user_id, name, severity, noted_at, notes, created_at
        FROM symptoms
        WHERE user_id = $1
        ORDER BY noted_at DESC
        LIMIT 300
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
    Json(body): Json<CreateSymptom>,
) -> Result<(StatusCode, Json<Symptom>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(insert(&state, user.id, &body).await?),
    ))
}

pub async fn insert(
    state: &AppState,
    user_id: Uuid,
    body: &CreateSymptom,
) -> Result<Symptom, AppError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::validation("A symptom or note title is required"));
    }
    if body
        .severity
        .is_some_and(|value| !(1..=10).contains(&value))
    {
        return Err(AppError::validation("Severity must be between 1 and 10"));
    }
    sqlx::query_as::<_, Symptom>(
        r#"
        INSERT INTO symptoms (user_id, name, severity, noted_at, notes)
        VALUES ($1,$2,$3,$4,$5)
        RETURNING id, user_id, name, severity, noted_at, notes, created_at
        "#,
    )
    .bind(user_id)
    .bind(name.chars().take(160).collect::<String>())
    .bind(body.severity)
    .bind(body.noted_at.unwrap_or_else(Utc::now))
    .bind(
        body.notes
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.chars().take(2000).collect::<String>()),
    )
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM symptoms WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Symptom not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
