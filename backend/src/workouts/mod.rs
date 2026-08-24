//! Workouts, exercises, and sets. Implemented in the lifestyle phase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub name: String,
    pub sets: Vec<ExerciseSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSet {
    pub repetitions: Option<i32>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub duration_seconds: Option<i32>,
}
