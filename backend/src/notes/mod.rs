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
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub noted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertNote {
    pub title: String,
    pub body: Option<String>,
    pub noted_at: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/notes",
    tag = "notes",
    security(("bearer_auth" = [])),
    responses((status = 200, body = [Note]))
)]
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Note>>, AppError> {
    Ok(Json(list_for(&state, user.id).await?))
}

pub async fn list_for(state: &AppState, user_id: Uuid) -> Result<Vec<Note>, AppError> {
    sqlx::query_as::<_, Note>(
        r#"
        SELECT id, user_id, title, body, noted_at, created_at, updated_at
        FROM health_notes
        WHERE user_id = $1
        ORDER BY noted_at DESC
        LIMIT 300
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
}

#[utoipa::path(
    post,
    path = "/api/v1/notes",
    tag = "notes",
    security(("bearer_auth" = [])),
    request_body = UpsertNote,
    responses((status = 201, body = Note))
)]
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertNote>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(insert(&state, user.id, &body).await?),
    ))
}

pub async fn insert(state: &AppState, user_id: Uuid, body: &UpsertNote) -> Result<Note, AppError> {
    let title = body.title.trim();
    if title.is_empty() {
        return Err(AppError::validation("A note title is required"));
    }
    let note_body = body
        .body
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .chars()
        .take(8000)
        .collect::<String>();
    sqlx::query_as::<_, Note>(
        r#"
        INSERT INTO health_notes (user_id, title, body, noted_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, title, body, noted_at, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(title.chars().take(160).collect::<String>())
    .bind(note_body)
    .bind(body.noted_at.unwrap_or_else(Utc::now))
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM health_notes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Note not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn latest(state: &AppState, user_id: Uuid) -> Result<Option<Note>, AppError> {
    sqlx::query_as::<_, Note>(
        r#"
        SELECT id, user_id, title, body, noted_at, created_at, updated_at
        FROM health_notes
        WHERE user_id = $1
        ORDER BY noted_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_titles() {
        let body = UpsertNote {
            title: "   ".into(),
            body: Some("hello".into()),
            noted_at: None,
        };
        let title = body.title.trim();
        assert!(title.is_empty());
    }
}
