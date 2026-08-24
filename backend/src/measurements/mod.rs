//! Generic health measurements (weight, blood pressure, vitals, ...).
//! CRUD is introduced in Phase 2. Types are defined now so clients can share contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String,
    pub value: f64,
    pub unit: String,
    pub measured_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
];
