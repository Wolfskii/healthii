use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::AuthUser};

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct ActivityEvent {
    pub id: Uuid,
    pub action: String,
    pub resource_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn record(state: &AppState, user_id: Uuid, action: &str, resource_type: Option<&str>) {
    if let Err(error) = sqlx::query(
        r#"
        INSERT INTO audit_events (user_id, action, resource_type)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(resource_type)
    .execute(&state.pool)
    .await
    {
        tracing::warn!(error = %error, "audit write failed");
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/activity",
    tag = "users",
    security(("bearer_auth" = [])),
    responses((status = 200, body = [ActivityEvent]))
)]
pub async fn list(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<Vec<ActivityEvent>>, AppError> {
    let rows = sqlx::query_as::<_, ActivityEvent>(
        r#"
        SELECT id, action, resource_type, created_at
        FROM audit_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(axum::Json(rows))
}

pub async fn purge_for_user(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM audit_events WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    Ok(())
}
