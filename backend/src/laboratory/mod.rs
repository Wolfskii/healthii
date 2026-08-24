//! Laboratory panels, biomarker definitions, and reference ranges.
//! Flexible by design: biomarker names are data, not hardcoded schema.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabTest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tested_on: NaiveDate,
    pub laboratory: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabResult {
    pub id: Uuid,
    pub lab_test_id: Uuid,
    pub biomarker_code: String,
    pub value: Option<f64>,
    pub value_text: Option<String>,
    pub unit: Option<String>,
    pub reference_low: Option<f64>,
    pub reference_high: Option<f64>,
    pub reference_text: Option<String>,
    pub status: LabStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabStatus {
    Low,
    Normal,
    High,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerDefinition {
    pub code: String,
    pub name: String,
    pub default_unit: Option<String>,
    pub category: Option<String>,
}
