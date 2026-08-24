//! Symptom logs and free-form health notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symptom {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub severity: Option<i16>,
    pub noted_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
