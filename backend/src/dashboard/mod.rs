use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{error::AppError, users::AuthUser};

const DISCLAIMER: &str = "Healthii is a personal health tracking and organization tool. It does not provide medical diagnosis or replace professional medical advice.";

#[derive(Serialize, ToSchema)]
pub struct DashboardResponse {
    pub mode: String,
    pub disclaimer: String,
    pub generated_at: DateTime<Utc>,
    pub widgets: Vec<DashboardWidget>,
}

#[derive(Serialize, ToSchema)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub empty: bool,
    pub summary: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard",
    tag = "dashboard",
    security(("bearer_auth" = [])),
    responses((status = 200, body = DashboardResponse))
)]
pub async fn get_dashboard(_user: AuthUser) -> Result<Json<DashboardResponse>, AppError> {
    Ok(Json(DashboardResponse {
        mode: "live".into(),
        disclaimer: DISCLAIMER.into(),
        generated_at: Utc::now(),
        widgets: vec![
            widget("weight", "Current weight", "metric"),
            widget("weight-trend", "Weight trend", "chart"),
            widget("labs", "Latest blood tests", "list"),
            widget("measurements", "Recent measurements", "list"),
            widget("blood-pressure", "Latest blood pressure", "metric"),
            widget("heart-rate", "Resting heart rate", "metric"),
            widget("workouts", "Recent workouts", "list"),
            widget("appointments", "Upcoming appointments", "list"),
            widget("documents", "Recent documents", "list"),
            widget("timeline", "Health timeline", "timeline"),
        ],
    }))
}

fn widget(id: &str, title: &str, kind: &str) -> DashboardWidget {
    DashboardWidget {
        id: id.into(),
        title: title.into(),
        kind: kind.into(),
        empty: true,
        summary: None,
    }
}
