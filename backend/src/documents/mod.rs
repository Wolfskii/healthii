//! User-uploaded medical documents. Bytes live behind the storage abstraction.
//! Routes are not exposed until authenticated ownership checks are complete.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    DoctorReport,
    BloodTest,
    Prescription,
    Imaging,
    Vaccination,
    Referral,
    DischargeSummary,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_key: String,
    pub document_type: DocumentType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub uploaded_at: DateTime<Utc>,
}

pub const ALLOWED_MIME_TYPES: &[&str] =
    &["application/pdf", "image/png", "image/jpeg", "image/webp"];
