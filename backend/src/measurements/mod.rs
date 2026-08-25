use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::AuthUser};

pub const MEASUREMENT_TYPES: &[&str] = &[
    "weight",
    "height",
    "bmi",
    "body_fat",
    "blood_pressure_systolic",
    "blood_pressure_diastolic",
    "resting_heart_rate",
    "heart_rate",
    "blood_glucose",
    "body_temperature",
    "oxygen_saturation",
    "waist_circumference",
    "sleep",
];

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Measurement {
    pub id: Uuid,
    pub user_id: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub kind: String,
    pub value: f64,
    pub unit: String,
    pub measured_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertMeasurement {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: f64,
    pub unit: String,
    pub measured_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BloodPressureRequest {
    pub systolic: f64,
    pub diastolic: f64,
    pub unit: Option<String>,
    pub measured_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BloodPressureResponse {
    pub systolic: Measurement,
    pub diastolic: Measurement,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/measurements",
    tag = "measurements",
    security(("bearer_auth" = [])),
    responses((status = 200, body = [Measurement]))
)]
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Measurement>>, AppError> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let rows = sqlx::query_as::<_, Measurement>(
        r#"
        SELECT id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        FROM measurements
        WHERE user_id = $1
          AND ($2::text IS NULL OR type = $2)
          AND ($3::timestamptz IS NULL OR measured_at >= $3)
          AND ($4::timestamptz IS NULL OR measured_at <= $4)
        ORDER BY measured_at DESC
        LIMIT $5
        "#,
    )
    .bind(user.id)
    .bind(query.kind.as_deref())
    .bind(query.from)
    .bind(query.to)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/measurements",
    tag = "measurements",
    security(("bearer_auth" = [])),
    request_body = UpsertMeasurement,
    responses((status = 201, body = Measurement))
)]
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertMeasurement>,
) -> Result<(StatusCode, Json<Measurement>), AppError> {
    let row = insert_measurement(&state, user.id, &body).await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    post,
    path = "/api/v1/measurements/blood-pressure",
    tag = "measurements",
    security(("bearer_auth" = [])),
    request_body = BloodPressureRequest,
    responses((status = 201, body = BloodPressureResponse))
)]
pub async fn create_blood_pressure(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<BloodPressureRequest>,
) -> Result<(StatusCode, Json<BloodPressureResponse>), AppError> {
    let measured_at = body.measured_at.unwrap_or_else(Utc::now);
    let unit = body.unit.unwrap_or_else(|| "mmHg".into());
    let source = body.source.unwrap_or_else(|| "manual".into());
    let systolic = insert_measurement(
        &state,
        user.id,
        &UpsertMeasurement {
            kind: "blood_pressure_systolic".into(),
            value: body.systolic,
            unit: unit.clone(),
            measured_at: Some(measured_at),
            source: Some(source.clone()),
            notes: body.notes.clone(),
        },
    )
    .await?;
    let diastolic = insert_measurement(
        &state,
        user.id,
        &UpsertMeasurement {
            kind: "blood_pressure_diastolic".into(),
            value: body.diastolic,
            unit,
            measured_at: Some(measured_at),
            source: Some(source),
            notes: body.notes,
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(BloodPressureResponse {
            systolic,
            diastolic,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/measurements/{id}",
    tag = "measurements",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Measurement id")),
    responses((status = 200, body = Measurement))
)]
pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Measurement>, AppError> {
    Ok(Json(owned(&state, user.id, id).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/measurements/{id}",
    tag = "measurements",
    security(("bearer_auth" = [])),
    request_body = UpsertMeasurement,
    responses((status = 200, body = Measurement))
)]
pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpsertMeasurement>,
) -> Result<Json<Measurement>, AppError> {
    let _ = owned(&state, user.id, id).await?;
    let kind = normalize_type(&body.kind)?;
    let unit = require_unit(&body.unit)?;
    validate_value(&kind, body.value, &unit)?;
    let measured_at = body.measured_at.unwrap_or_else(Utc::now);
    let source = normalize_source(body.source.as_deref());
    let notes = normalize_notes(body.notes.as_deref());
    let row = sqlx::query_as::<_, Measurement>(
        r#"
        UPDATE measurements
        SET type = $3, value = $4, unit = $5, measured_at = $6, source = $7, notes = $8, updated_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(&kind)
    .bind(body.value)
    .bind(&unit)
    .bind(measured_at)
    .bind(&source)
    .bind(notes)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Measurement not found"))?;
    Ok(Json(row))
}

#[utoipa::path(
    delete,
    path = "/api/v1/measurements/{id}",
    tag = "measurements",
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Deleted"))
)]
pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let _ = owned(&state, user.id, id).await?;
    sqlx::query("DELETE FROM measurements WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn insert_measurement(
    state: &AppState,
    user_id: Uuid,
    body: &UpsertMeasurement,
) -> Result<Measurement, AppError> {
    let kind = normalize_type(&body.kind)?;
    let unit = require_unit(&body.unit)?;
    validate_value(&kind, body.value, &unit)?;
    let measured_at = body.measured_at.unwrap_or_else(Utc::now);
    let source = normalize_source(body.source.as_deref());
    let notes = normalize_notes(body.notes.as_deref());
    sqlx::query_as::<_, Measurement>(
        r#"
        INSERT INTO measurements (user_id, type, value, unit, measured_at, source, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(&kind)
    .bind(body.value)
    .bind(&unit)
    .bind(measured_at)
    .bind(&source)
    .bind(notes)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn owned(state: &AppState, user_id: Uuid, id: Uuid) -> Result<Measurement, AppError> {
    sqlx::query_as::<_, Measurement>(
        r#"
        SELECT id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        FROM measurements
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Measurement not found"))
}

pub async fn latest(
    state: &AppState,
    user_id: Uuid,
    kind: &str,
) -> Result<Option<Measurement>, AppError> {
    sqlx::query_as::<_, Measurement>(
        r#"
        SELECT id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        FROM measurements
        WHERE user_id = $1 AND type = $2
        ORDER BY measured_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(kind)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::from)
}

pub async fn since(
    state: &AppState,
    user_id: Uuid,
    kind: &str,
    from: DateTime<Utc>,
) -> Result<Vec<Measurement>, AppError> {
    sqlx::query_as::<_, Measurement>(
        r#"
        SELECT id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        FROM measurements
        WHERE user_id = $1 AND type = $2 AND measured_at >= $3
        ORDER BY measured_at ASC
        "#,
    )
    .bind(user_id)
    .bind(kind)
    .bind(from)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
}

pub fn trend_delta(points: &[Measurement]) -> Option<f64> {
    let first = points.first()?;
    let last = points.last()?;
    if points.len() < 2 {
        return None;
    }
    Some(last.value - first.value)
}

pub fn days_ago(days: i64) -> DateTime<Utc> {
    Utc::now() - Duration::days(days)
}

fn normalize_type(kind: &str) -> Result<String, AppError> {
    let kind = kind.trim().to_ascii_lowercase();
    if MEASUREMENT_TYPES.contains(&kind.as_str()) {
        Ok(kind)
    } else {
        Err(AppError::validation(
            "Unknown measurement type. Add a supported type rather than inventing schema.",
        ))
    }
}

fn require_unit(unit: &str) -> Result<String, AppError> {
    let unit = unit.trim();
    if unit.is_empty() || unit.len() > 24 {
        return Err(AppError::validation("A unit is required"));
    }
    Ok(unit.to_string())
}

fn normalize_source(source: Option<&str>) -> String {
    source
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("manual")
        .chars()
        .take(40)
        .collect()
}

fn normalize_notes(notes: Option<&str>) -> Option<String> {
    notes
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(2000).collect())
}

pub fn scale_sleep_import(value: f64, unit: &str) -> (f64, String) {
    let lower = unit.to_ascii_lowercase();
    let hours = if lower.contains("min") {
        value / 60.0
    } else if lower.contains("sec") || lower == "s" {
        value / 3600.0
    } else if value > 24.0 && value <= 1440.0 && !lower.contains('h') {
        value / 60.0
    } else {
        value
    };
    (hours, "h".into())
}

fn validate_value(kind: &str, value: f64, unit: &str) -> Result<(), AppError> {
    if !value.is_finite() {
        return Err(AppError::validation("Invalid measurement value"));
    }
    let unit = unit.to_ascii_lowercase();
    let ok = match kind {
        "weight" => (1.0..=500.0).contains(&value),
        "height" => (20.0..=280.0).contains(&value),
        "bmi" => (5.0..=90.0).contains(&value),
        "body_fat" => (1.0..=80.0).contains(&value),
        "blood_pressure_systolic" => (50.0..=300.0).contains(&value),
        "blood_pressure_diastolic" => (20.0..=200.0).contains(&value),
        "resting_heart_rate" | "heart_rate" => (20.0..=250.0).contains(&value),
        "blood_glucose" => {
            if unit.contains("mg") {
                (20.0..=600.0).contains(&value)
            } else {
                (0.5..=40.0).contains(&value)
            }
        }
        "body_temperature" => (30.0..=45.0).contains(&value) || (86.0..=113.0).contains(&value),
        "oxygen_saturation" => (50.0..=100.0).contains(&value),
        "waist_circumference" => (20.0..=300.0).contains(&value),
        "sleep" => {
            if unit.contains("min") {
                (0.0..=1440.0).contains(&value)
            } else {
                (0.0..=24.0).contains(&value)
            }
        }
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(AppError::validation("Invalid measurement value"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_types() {
        assert!(normalize_type("mood").is_err());
        assert_eq!(normalize_type("Weight").unwrap(), "weight");
    }

    #[test]
    fn rejects_non_finite_values() {
        assert!(validate_value("weight", f64::NAN, "kg").is_err());
        assert!(validate_value("weight", 82.4, "kg").is_ok());
        assert!(validate_value("blood_glucose", 95.0, "mg/dL").is_ok());
        assert!(validate_value("sleep", 7.5, "h").is_ok());
        assert!(validate_value("sleep", 450.0, "min").is_ok());
        assert!(validate_value("sleep", 30.0, "h").is_err());
    }
}
