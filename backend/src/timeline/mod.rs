use axum::{extract::Query, extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::AuthUser};

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub kind: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TimelineItem {
    pub id: Uuid,
    pub kind: String,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TimelineResponse {
    pub items: Vec<TimelineItem>,
}

pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<TimelineResponse>, AppError> {
    let limit = query.limit.unwrap_or(80).clamp(1, 200) as usize;
    let kind = query.kind.as_deref().unwrap_or("all");
    let mut items = Vec::new();

    if matches!(kind, "all" | "measurements") {
        let rows = sqlx::query_as::<_, (Uuid, String, f64, String, DateTime<Utc>)>(
            r#"
            SELECT id, type, value, unit, measured_at
            FROM measurements
            WHERE user_id = $1
            ORDER BY measured_at DESC
            LIMIT 80
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, type_name, value, unit, occurred_at) in rows {
            items.push(TimelineItem {
                id,
                kind: "measurements".into(),
                occurred_at,
                title: type_name.replace('_', " "),
                detail: Some(format!("{value} {unit}")),
            });
        }
    }

    if matches!(kind, "all" | "labs") {
        let rows = sqlx::query_as::<_, (Uuid, chrono::NaiveDate, Option<String>, i64)>(
            r#"
            SELECT t.id, t.tested_on, t.laboratory, COUNT(r.id)::bigint
            FROM lab_tests t
            LEFT JOIN lab_results r ON r.lab_test_id = t.id
            WHERE t.user_id = $1
            GROUP BY t.id
            ORDER BY t.tested_on DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, tested_on, laboratory, count) in rows {
            items.push(TimelineItem {
                id,
                kind: "labs".into(),
                occurred_at: tested_on
                    .and_hms_opt(8, 0, 0)
                    .map(|d| d.and_utc())
                    .unwrap_or_else(Utc::now),
                title: "Blood test".into(),
                detail: Some(format!(
                    "{} · {count} biomarkers",
                    laboratory.unwrap_or_else(|| "Laboratory".into())
                )),
            });
        }
    }

    if matches!(kind, "all" | "documents") {
        let rows = sqlx::query_as::<_, (Uuid, String, Option<String>, DateTime<Utc>)>(
            r#"
            SELECT id, filename, title, uploaded_at
            FROM documents
            WHERE user_id = $1
            ORDER BY uploaded_at DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, filename, title, occurred_at) in rows {
            items.push(TimelineItem {
                id,
                kind: "documents".into(),
                occurred_at,
                title: title.unwrap_or(filename),
                detail: Some("Document".into()),
            });
        }
    }

    if matches!(kind, "all" | "workouts") {
        let rows = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, Option<i32>)>(
            r#"
            SELECT id, workout_type, started_at, duration_seconds
            FROM workouts
            WHERE user_id = $1
            ORDER BY started_at DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, workout_type, occurred_at, duration) in rows {
            items.push(TimelineItem {
                id,
                kind: "workouts".into(),
                occurred_at,
                title: format!("Workout · {workout_type}"),
                detail: duration.map(|seconds| format!("{} min", seconds / 60)),
            });
        }
    }

    if matches!(kind, "all" | "appointments") {
        let rows = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, Option<String>)>(
            r#"
            SELECT id, title, starts_at, provider
            FROM appointments
            WHERE user_id = $1
            ORDER BY starts_at DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, title, occurred_at, provider) in rows {
            items.push(TimelineItem {
                id,
                kind: "appointments".into(),
                occurred_at,
                title,
                detail: provider,
            });
        }
    }

    if matches!(kind, "all" | "medications") {
        let rows = sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
            r#"
            SELECT id, name, kind, created_at
            FROM medications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, name, med_kind, occurred_at) in rows {
            items.push(TimelineItem {
                id,
                kind: "medications".into(),
                occurred_at,
                title: name,
                detail: Some(med_kind),
            });
        }
    }

    if matches!(kind, "all" | "symptoms") {
        let rows = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, Option<i16>)>(
            r#"
            SELECT id, name, noted_at, severity
            FROM symptoms
            WHERE user_id = $1
            ORDER BY noted_at DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, name, occurred_at, severity) in rows {
            items.push(TimelineItem {
                id,
                kind: "symptoms".into(),
                occurred_at,
                title: name,
                detail: severity.map(|value| format!("Severity {value}/10")),
            });
        }
    }

    if matches!(kind, "all" | "notes") {
        let rows = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, String)>(
            r#"
            SELECT id, title, noted_at, body
            FROM health_notes
            WHERE user_id = $1
            ORDER BY noted_at DESC
            LIMIT 40
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
        for (id, title, occurred_at, body) in rows {
            let detail = body.chars().take(80).collect::<String>();
            items.push(TimelineItem {
                id,
                kind: "notes".into(),
                occurred_at,
                title,
                detail: if detail.is_empty() {
                    None
                } else {
                    Some(detail)
                },
            });
        }
    }

    items.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
    items.truncate(limit);
    Ok(Json(TimelineResponse { items }))
}
