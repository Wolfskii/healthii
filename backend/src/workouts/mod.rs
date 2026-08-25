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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutType {
    Strength,
    Running,
    Cycling,
    Walking,
    Swimming,
    Sports,
    Mobility,
    Other,
}

impl WorkoutType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strength => "strength",
            Self::Running => "running",
            Self::Cycling => "cycling",
            Self::Walking => "walking",
            Self::Swimming => "swimming",
            Self::Sports => "sports",
            Self::Mobility => "mobility",
            Self::Other => "other",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "strength" => Ok(Self::Strength),
            "running" => Ok(Self::Running),
            "cycling" => Ok(Self::Cycling),
            "walking" => Ok(Self::Walking),
            "swimming" => Ok(Self::Swimming),
            "sports" => Ok(Self::Sports),
            "mobility" => Ok(Self::Mobility),
            "other" => Ok(Self::Other),
            _ => Err(AppError::validation("Unknown workout type")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct ExerciseSet {
    pub repetitions: Option<i32>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Exercise {
    pub name: String,
    #[serde(default)]
    pub sets: Vec<ExerciseSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Workout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workout_type: WorkoutType,
    pub started_at: DateTime<Utc>,
    pub duration_seconds: Option<i32>,
    pub distance: Option<f64>,
    pub distance_unit: Option<String>,
    pub calories: Option<i32>,
    pub notes: Option<String>,
    pub exercises: Vec<Exercise>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct WorkoutRow {
    id: Uuid,
    user_id: Uuid,
    workout_type: String,
    started_at: DateTime<Utc>,
    duration_seconds: Option<i32>,
    distance: Option<f64>,
    distance_unit: Option<String>,
    calories: Option<i32>,
    notes: Option<String>,
    exercises: sqlx::types::Json<Vec<Exercise>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<WorkoutRow> for Workout {
    type Error = AppError;

    fn try_from(row: WorkoutRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            user_id: row.user_id,
            workout_type: WorkoutType::parse(&row.workout_type)?,
            started_at: row.started_at,
            duration_seconds: row.duration_seconds,
            distance: row.distance,
            distance_unit: row.distance_unit,
            calories: row.calories,
            notes: row.notes,
            exercises: row.exercises.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertWorkout {
    pub workout_type: WorkoutType,
    pub started_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub distance: Option<f64>,
    pub distance_unit: Option<String>,
    pub calories: Option<i32>,
    pub notes: Option<String>,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Workout>>, AppError> {
    let rows = sqlx::query_as::<_, WorkoutRow>(
        r#"
        SELECT id, user_id, workout_type, started_at, duration_seconds, distance, distance_unit,
               calories, notes, exercises, created_at, updated_at
        FROM workouts
        WHERE user_id = $1
        ORDER BY started_at DESC
        LIMIT 200
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(Workout::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
}

pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertWorkout>,
) -> Result<(StatusCode, Json<Workout>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(insert(&state, user.id, &body).await?),
    ))
}

pub async fn insert(
    state: &AppState,
    user_id: Uuid,
    body: &UpsertWorkout,
) -> Result<Workout, AppError> {
    validate(body)?;
    let started_at = body.started_at.unwrap_or_else(Utc::now);
    let row = sqlx::query_as::<_, WorkoutRow>(
        r#"
        INSERT INTO workouts (
            user_id, workout_type, started_at, duration_seconds, distance, distance_unit,
            calories, notes, exercises
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        RETURNING id, user_id, workout_type, started_at, duration_seconds, distance, distance_unit,
                  calories, notes, exercises, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(body.workout_type.as_str())
    .bind(started_at)
    .bind(body.duration_seconds)
    .bind(body.distance)
    .bind(trim_opt(body.distance_unit.as_deref(), 16))
    .bind(body.calories)
    .bind(trim_opt(body.notes.as_deref(), 2000))
    .bind(sqlx::types::Json(&body.exercises))
    .fetch_one(&state.pool)
    .await?;
    Workout::try_from(row)
}

pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Workout>, AppError> {
    Ok(Json(owned(&state, user.id, id).await?))
}

pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpsertWorkout>,
) -> Result<Json<Workout>, AppError> {
    let _ = owned(&state, user.id, id).await?;
    validate(&body)?;
    let started_at = body.started_at.unwrap_or_else(Utc::now);
    let row = sqlx::query_as::<_, WorkoutRow>(
        r#"
        UPDATE workouts
        SET workout_type = $3, started_at = $4, duration_seconds = $5, distance = $6,
            distance_unit = $7, calories = $8, notes = $9, exercises = $10, updated_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, workout_type, started_at, duration_seconds, distance, distance_unit,
                  calories, notes, exercises, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(body.workout_type.as_str())
    .bind(started_at)
    .bind(body.duration_seconds)
    .bind(body.distance)
    .bind(trim_opt(body.distance_unit.as_deref(), 16))
    .bind(body.calories)
    .bind(trim_opt(body.notes.as_deref(), 2000))
    .bind(sqlx::types::Json(&body.exercises))
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Workout not found"))?;
    Ok(Json(Workout::try_from(row)?))
}

pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let _ = owned(&state, user.id, id).await?;
    sqlx::query("DELETE FROM workouts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn recent(state: &AppState, user_id: Uuid, limit: i64) -> Result<Vec<Workout>, AppError> {
    let rows = sqlx::query_as::<_, WorkoutRow>(
        r#"
        SELECT id, user_id, workout_type, started_at, duration_seconds, distance, distance_unit,
               calories, notes, exercises, created_at, updated_at
        FROM workouts
        WHERE user_id = $1
        ORDER BY started_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter().map(Workout::try_from).collect()
}

async fn owned(state: &AppState, user_id: Uuid, id: Uuid) -> Result<Workout, AppError> {
    let row = sqlx::query_as::<_, WorkoutRow>(
        r#"
        SELECT id, user_id, workout_type, started_at, duration_seconds, distance, distance_unit,
               calories, notes, exercises, created_at, updated_at
        FROM workouts
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Workout not found"))?;
    Workout::try_from(row)
}

fn validate(body: &UpsertWorkout) -> Result<(), AppError> {
    for exercise in &body.exercises {
        if exercise.name.trim().is_empty() {
            return Err(AppError::validation("Exercise name is required"));
        }
    }
    if body
        .distance
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(AppError::validation("Distance must be a positive number"));
    }
    Ok(())
}

fn trim_opt(value: Option<&str>, max: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max).collect())
}
