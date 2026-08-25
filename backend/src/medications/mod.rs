use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::AuthUser};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Medication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub kind: String,
    pub dosage: Option<String>,
    pub schedule: Option<String>,
    pub started_on: Option<NaiveDate>,
    pub ended_on: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertMedication {
    pub name: String,
    pub kind: Option<String>,
    pub dosage: Option<String>,
    pub schedule: Option<String>,
    pub started_on: Option<NaiveDate>,
    pub ended_on: Option<NaiveDate>,
    pub notes: Option<String>,
}

pub async fn active(
    state: &AppState,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<Medication>, AppError> {
    sqlx::query_as::<_, Medication>(
        r#"
        SELECT id, user_id, name, kind, dosage, schedule, started_on, ended_on, notes, created_at, updated_at
        FROM medications
        WHERE user_id = $1
          AND (ended_on IS NULL OR ended_on >= CURRENT_DATE)
        ORDER BY name
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
) -> Result<Json<Vec<Medication>>, AppError> {
    let rows = sqlx::query_as::<_, Medication>(
        r#"
        SELECT id, user_id, name, kind, dosage, schedule, started_on, ended_on, notes, created_at, updated_at
        FROM medications
        WHERE user_id = $1
        ORDER BY name
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
    Json(body): Json<UpsertMedication>,
) -> Result<(StatusCode, Json<Medication>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(insert(&state, user.id, &body).await?),
    ))
}

pub async fn insert(
    state: &AppState,
    user_id: Uuid,
    body: &UpsertMedication,
) -> Result<Medication, AppError> {
    let name = require_name(&body.name)?;
    let kind = normalize_kind(body.kind.as_deref())?;
    sqlx::query_as::<_, Medication>(
        r#"
        INSERT INTO medications (user_id, name, kind, dosage, schedule, started_on, ended_on, notes)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        RETURNING id, user_id, name, kind, dosage, schedule, started_on, ended_on, notes, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(&name)
    .bind(&kind)
    .bind(trim_opt(body.dosage.as_deref(), 80))
    .bind(trim_opt(body.schedule.as_deref(), 160))
    .bind(body.started_on)
    .bind(body.ended_on)
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
    let result = sqlx::query("DELETE FROM medications WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Medication not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn require_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::validation("A name is required"));
    }
    Ok(name.chars().take(160).collect())
}

fn normalize_kind(kind: Option<&str>) -> Result<String, AppError> {
    match kind.unwrap_or("medication").trim() {
        "medication" | "supplement" => Ok(kind.unwrap_or("medication").trim().into()),
        _ => Err(AppError::validation(
            "Kind must be medication or supplement",
        )),
    }
}

fn trim_opt(value: Option<&str>, max: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max).collect())
}
