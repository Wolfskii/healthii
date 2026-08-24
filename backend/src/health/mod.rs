use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{db, error::AppError, state::AppState};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "ops",
    responses((status = 200, description = "Process is running", body = HealthResponse))
)]
pub async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "healthii-backend".into(),
    })
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "ops",
    responses(
        (status = 200, description = "Dependencies are reachable", body = HealthResponse),
        (status = 503, description = "A dependency is unavailable")
    )
)]
pub async fn ready(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    db::ping(&state.pool).await?;
    Ok(Json(HealthResponse {
        status: "ready".into(),
        service: "healthii-backend".into(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn live_reports_ok() {
        let response = live().await;
        assert_eq!(response.status, "ok");
    }
}
