use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, users::AuthUser};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Profile {
    pub user_id: Uuid,
    pub height_cm: Option<f64>,
    pub blood_type: Option<String>,
    pub allergies: Option<String>,
    pub medical_history: Option<String>,
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>,
    pub unit_system: String,
    pub weight_unit: String,
    pub goal_weight: Option<f64>,
    pub goal_weight_unit: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfile {
    pub height_cm: Option<f64>,
    pub blood_type: Option<String>,
    pub allergies: Option<String>,
    pub medical_history: Option<String>,
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>,
    pub unit_system: Option<String>,
    pub weight_unit: Option<String>,
    pub goal_weight: Option<f64>,
    pub goal_weight_unit: Option<String>,
}

pub async fn get(user: AuthUser, State(state): State<AppState>) -> Result<Json<Profile>, AppError> {
    Ok(Json(load(&state, user.id).await?))
}

pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfile>,
) -> Result<Json<Profile>, AppError> {
    if body
        .height_cm
        .is_some_and(|value| !value.is_finite() || !(20.0..=280.0).contains(&value))
    {
        return Err(AppError::validation(
            "Height must be a realistic centimeter value",
        ));
    }
    if body
        .goal_weight
        .is_some_and(|value| !value.is_finite() || !(20.0..=400.0).contains(&value))
    {
        return Err(AppError::validation(
            "Goal weight must be a realistic value",
        ));
    }
    let unit_system = match body.unit_system.as_deref().unwrap_or("metric") {
        "metric" | "imperial" => body.unit_system.unwrap_or_else(|| "metric".into()),
        _ => {
            return Err(AppError::validation(
                "Unit system must be metric or imperial",
            ))
        }
    };
    let weight_unit = match body.weight_unit.as_deref().unwrap_or("kg") {
        "kg" | "lb" => body.weight_unit.unwrap_or_else(|| "kg".into()),
        _ => return Err(AppError::validation("Weight unit must be kg or lb")),
    };
    let goal_weight_unit = match body.goal_weight_unit.as_deref() {
        None | Some("") => None,
        Some("kg" | "lb") => body.goal_weight_unit.clone(),
        _ => return Err(AppError::validation("Goal weight unit must be kg or lb")),
    };
    sqlx::query("INSERT INTO profiles (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING")
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    let row = sqlx::query_as::<_, Profile>(
        r#"
        UPDATE profiles
        SET height_cm = $2,
            blood_type = $3,
            allergies = $4,
            medical_history = $5,
            emergency_name = $6,
            emergency_phone = $7,
            unit_system = $8,
            weight_unit = $9,
            goal_weight = $10,
            goal_weight_unit = $11,
            updated_at = now()
        WHERE user_id = $1
        RETURNING user_id, height_cm, blood_type, allergies, medical_history,
                  emergency_name, emergency_phone, unit_system, weight_unit,
                  goal_weight, goal_weight_unit, updated_at
        "#,
    )
    .bind(user.id)
    .bind(body.height_cm)
    .bind(trim_opt(body.blood_type.as_deref(), 16))
    .bind(trim_opt(body.allergies.as_deref(), 1000))
    .bind(trim_opt(body.medical_history.as_deref(), 4000))
    .bind(trim_opt(body.emergency_name.as_deref(), 120))
    .bind(trim_opt(body.emergency_phone.as_deref(), 40))
    .bind(&unit_system)
    .bind(&weight_unit)
    .bind(body.goal_weight)
    .bind(goal_weight_unit.as_deref())
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

pub async fn load(state: &AppState, user_id: Uuid) -> Result<Profile, AppError> {
    sqlx::query("INSERT INTO profiles (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING")
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    sqlx::query_as::<_, Profile>(
        r#"
        SELECT user_id, height_cm, blood_type, allergies, medical_history,
               emergency_name, emergency_phone, unit_system, weight_unit,
               goal_weight, goal_weight_unit, updated_at
        FROM profiles
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)
}

fn trim_opt(value: Option<&str>, max: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max).collect())
}
