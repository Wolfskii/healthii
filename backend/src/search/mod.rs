use axum::{extract::Query, extract::State, Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::AppState,
    timeline::{TimelineItem, TimelineResponse},
    users::AuthUser,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    tag = "dashboard",
    security(("bearer_auth" = [])),
    params(("q" = String, Query, description = "Case-insensitive text, at least two characters")),
    responses((status = 200, body = TimelineResponse))
)]
pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<TimelineResponse>, AppError> {
    let needle = query.q.unwrap_or_default();
    let needle = needle.trim();
    if needle.chars().count() < 2 {
        return Ok(Json(TimelineResponse { items: Vec::new() }));
    }
    let limit = query.limit.unwrap_or(40).clamp(1, 80) as usize;
    let mut items = Vec::new();

    let measurements = sqlx::query_as::<_, (Uuid, String, f64, String, DateTime<Utc>)>(
        r#"
        SELECT id, type, value, unit, measured_at
        FROM measurements
        WHERE user_id = $1
          AND strpos(lower(type || ' ' || coalesce(notes, '') || ' ' || unit), lower($2)) > 0
        ORDER BY measured_at DESC
        LIMIT 40
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, type_name, value, unit, occurred_at) in measurements {
        items.push(TimelineItem {
            id,
            kind: "measurements".into(),
            occurred_at,
            title: type_name.replace('_', " "),
            detail: Some(format!("{value} {unit}")),
        });
    }

    let labs = sqlx::query_as::<_, (Uuid, chrono::NaiveDate, Option<String>)>(
        r#"
        SELECT DISTINCT t.id, t.tested_on, t.laboratory
        FROM lab_tests t
        LEFT JOIN lab_results r ON r.lab_test_id = t.id
        WHERE t.user_id = $1
          AND strpos(
                lower(
                    coalesce(t.laboratory, '') || ' ' || coalesce(t.notes, '') || ' ' ||
                    coalesce(r.biomarker_code, '') || ' ' || coalesce(r.name, '')
                ),
                lower($2)
              ) > 0
        ORDER BY t.tested_on DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, tested_on, laboratory) in labs {
        items.push(TimelineItem {
            id,
            kind: "labs".into(),
            occurred_at: tested_on
                .and_hms_opt(8, 0, 0)
                .map(|d| d.and_utc())
                .unwrap_or_else(Utc::now),
            title: "Blood test".into(),
            detail: laboratory,
        });
    }

    let documents = sqlx::query_as::<_, (Uuid, String, Option<String>, DateTime<Utc>)>(
        r#"
        SELECT id, filename, title, uploaded_at
        FROM documents
        WHERE user_id = $1
          AND strpos(
                lower(filename || ' ' || coalesce(title, '') || ' ' || coalesce(description, '') || ' ' || document_type),
                lower($2)
              ) > 0
        ORDER BY uploaded_at DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, filename, title, occurred_at) in documents {
        items.push(TimelineItem {
            id,
            kind: "documents".into(),
            occurred_at,
            title: title.unwrap_or(filename),
            detail: Some("Document".into()),
        });
    }

    let workouts = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        r#"
        SELECT id, workout_type, started_at
        FROM workouts
        WHERE user_id = $1
          AND strpos(lower(workout_type || ' ' || coalesce(notes, '')), lower($2)) > 0
        ORDER BY started_at DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, workout_type, occurred_at) in workouts {
        items.push(TimelineItem {
            id,
            kind: "workouts".into(),
            occurred_at,
            title: format!("Workout · {workout_type}"),
            detail: None,
        });
    }

    let medications = sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
        r#"
        SELECT id, name, kind, created_at
        FROM medications
        WHERE user_id = $1
          AND strpos(lower(name || ' ' || coalesce(dosage, '') || ' ' || coalesce(notes, '')), lower($2)) > 0
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, name, med_kind, occurred_at) in medications {
        items.push(TimelineItem {
            id,
            kind: "medications".into(),
            occurred_at,
            title: name,
            detail: Some(med_kind),
        });
    }

    let symptoms = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, Option<String>)>(
        r#"
        SELECT id, name, noted_at, notes
        FROM symptoms
        WHERE user_id = $1
          AND strpos(lower(name || ' ' || coalesce(notes, '')), lower($2)) > 0
        ORDER BY noted_at DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, name, occurred_at, notes) in symptoms {
        items.push(TimelineItem {
            id,
            kind: "symptoms".into(),
            occurred_at,
            title: name,
            detail: notes,
        });
    }

    let appointments = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, Option<String>)>(
        r#"
        SELECT id, title, starts_at, provider
        FROM appointments
        WHERE user_id = $1
          AND strpos(
                lower(title || ' ' || coalesce(provider, '') || ' ' || coalesce(location, '') || ' ' || coalesce(notes, '')),
                lower($2)
              ) > 0
        ORDER BY starts_at DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, title, occurred_at, provider) in appointments {
        items.push(TimelineItem {
            id,
            kind: "appointments".into(),
            occurred_at,
            title,
            detail: provider,
        });
    }

    let notes = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, String)>(
        r#"
        SELECT id, title, noted_at, body
        FROM health_notes
        WHERE user_id = $1
          AND strpos(lower(title || ' ' || body), lower($2)) > 0
        ORDER BY noted_at DESC
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(needle)
    .fetch_all(&state.pool)
    .await?;
    for (id, title, occurred_at, body) in notes {
        items.push(TimelineItem {
            id,
            kind: "notes".into(),
            occurred_at,
            title,
            detail: Some(body.chars().take(80).collect()),
        });
    }

    items.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
    items.truncate(limit);
    Ok(Json(TimelineResponse { items }))
}
