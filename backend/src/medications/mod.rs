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

use crate::{
    error::AppError,
    state::AppState,
    users::{self, AuthUser},
};

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
    /// Derived by the server: started (or no start date) and not yet ended, in the user's timezone.
    pub active: bool,
}

/// Columns for every query that returns a [`Medication`]. `$today` is the
/// placeholder bound to the user's local date. A medication that ends today
/// counts as finished, so "stop today" removes it at once.
macro_rules! medication_columns {
    ($today:literal) => {
        concat!(
            "id, user_id, name, kind, dosage, schedule, started_on, ended_on, notes, created_at, updated_at, ",
            "((ended_on IS NULL OR ended_on > ", $today, ") ",
            "AND (started_on IS NULL OR started_on <= ", $today, ")) AS active"
        )
    };
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
    sqlx::query_as::<_, Medication>(concat!(
        "SELECT ",
        medication_columns!("$3"),
        " FROM medications
          WHERE user_id = $1
            AND (ended_on IS NULL OR ended_on > $3)
            AND (started_on IS NULL OR started_on <= $3)
          ORDER BY name
          LIMIT $2",
    ))
    .bind(user_id)
    .bind(limit)
    .bind(users::local_today(state, user_id).await)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_for(state: &AppState, user_id: Uuid) -> Result<Vec<Medication>, AppError> {
    sqlx::query_as::<_, Medication>(concat!(
        "SELECT ",
        medication_columns!("$2"),
        " FROM medications WHERE user_id = $1 ORDER BY name",
    ))
    .bind(user_id)
    .bind(users::local_today(state, user_id).await)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Medication>>, AppError> {
    Ok(Json(list_for(&state, user.id).await?))
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
    let clean = validate(body)?;
    sqlx::query_as::<_, Medication>(concat!(
        "INSERT INTO medications (user_id, name, kind, dosage, schedule, started_on, ended_on, notes)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         RETURNING ",
        medication_columns!("$9"),
    ))
    .bind(user_id)
    .bind(&clean.name)
    .bind(&clean.kind)
    .bind(&clean.dosage)
    .bind(&clean.schedule)
    .bind(clean.started_on)
    .bind(clean.ended_on)
    .bind(&clean.notes)
    .bind(users::local_today(state, user_id).await)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)
}

#[utoipa::path(
    put,
    path = "/api/v1/medications/{id}",
    tag = "medications",
    security(("bearer_auth" = [])),
    request_body = UpsertMedication,
    responses(
        (status = 200, description = "Updated", body = Medication),
        (status = 404, description = "Not found for this user", body = crate::error::ErrorBody)
    )
)]
pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpsertMedication>,
) -> Result<Json<Medication>, AppError> {
    let clean = validate(&body)?;
    let row = sqlx::query_as::<_, Medication>(concat!(
        "UPDATE medications
         SET name = $3, kind = $4, dosage = $5, schedule = $6, started_on = $7, ended_on = $8,
             notes = $9, updated_at = now()
         WHERE id = $1 AND user_id = $2
         RETURNING ",
        medication_columns!("$10"),
    ))
    .bind(id)
    .bind(user.id)
    .bind(&clean.name)
    .bind(&clean.kind)
    .bind(&clean.dosage)
    .bind(&clean.schedule)
    .bind(clean.started_on)
    .bind(clean.ended_on)
    .bind(&clean.notes)
    .bind(users::local_today(&state, user.id).await)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Medication not found"))?;
    Ok(Json(row))
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

struct CleanMedication {
    name: String,
    kind: String,
    dosage: Option<String>,
    schedule: Option<String>,
    started_on: Option<NaiveDate>,
    ended_on: Option<NaiveDate>,
    notes: Option<String>,
}

fn validate(body: &UpsertMedication) -> Result<CleanMedication, AppError> {
    if let (Some(started), Some(ended)) = (body.started_on, body.ended_on) {
        if ended < started {
            return Err(AppError::validation(
                "End date cannot be before the start date",
            ));
        }
    }
    Ok(CleanMedication {
        name: require_name(&body.name)?,
        kind: normalize_kind(body.kind.as_deref())?,
        dosage: trim_opt(body.dosage.as_deref(), 80),
        schedule: trim_opt(body.schedule.as_deref(), 160),
        started_on: body.started_on,
        ended_on: body.ended_on,
        notes: trim_opt(body.notes.as_deref(), 2000),
    })
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
        kind @ ("medication" | "supplement") => Ok(kind.into()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn body(started_on: Option<&str>, ended_on: Option<&str>) -> UpsertMedication {
        UpsertMedication {
            name: "  Vitamin D  ".into(),
            kind: Some("supplement".into()),
            dosage: Some(" ".into()),
            schedule: None,
            started_on: started_on.map(|value| value.parse().unwrap()),
            ended_on: ended_on.map(|value| value.parse().unwrap()),
            notes: None,
        }
    }

    #[test]
    fn validate_trims_and_drops_blank_fields() {
        let clean = validate(&body(None, None)).unwrap();
        assert_eq!(clean.name, "Vitamin D");
        assert_eq!(clean.kind, "supplement");
        assert_eq!(clean.dosage, None);
    }

    #[test]
    fn validate_rejects_end_before_start() {
        assert!(validate(&body(Some("2026-03-10"), Some("2026-03-01"))).is_err());
        assert!(validate(&body(Some("2026-03-10"), Some("2026-03-10"))).is_ok());
        assert!(validate(&body(None, Some("2026-03-01"))).is_ok());
    }

    #[test]
    fn validate_rejects_unknown_kind() {
        let mut row = body(None, None);
        row.kind = Some("candy".into());
        assert!(validate(&row).is_err());
    }
}
