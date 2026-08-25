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

use crate::{error::AppError, state::AppState, users::AuthUser};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LabStatus {
    Low,
    Normal,
    High,
    Critical,
    Unknown,
}

impl LabStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Critical => "critical",
            Self::Unknown => "unknown",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "low" => Self::Low,
            "normal" => Self::Normal,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BiomarkerDefinition {
    pub code: String,
    pub name: String,
    pub default_unit: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LabResult {
    pub id: Uuid,
    pub lab_test_id: Uuid,
    pub biomarker_code: String,
    pub name: Option<String>,
    pub value: Option<f64>,
    pub value_text: Option<String>,
    pub unit: Option<String>,
    pub reference_low: Option<f64>,
    pub reference_high: Option<f64>,
    pub reference_text: Option<String>,
    pub status: LabStatus,
}

#[derive(Debug, Clone, FromRow)]
struct LabResultRow {
    id: Uuid,
    lab_test_id: Uuid,
    biomarker_code: String,
    name: Option<String>,
    value: Option<f64>,
    value_text: Option<String>,
    unit: Option<String>,
    reference_low: Option<f64>,
    reference_high: Option<f64>,
    reference_text: Option<String>,
    status: String,
}

impl From<LabResultRow> for LabResult {
    fn from(row: LabResultRow) -> Self {
        Self {
            id: row.id,
            lab_test_id: row.lab_test_id,
            biomarker_code: row.biomarker_code,
            name: row.name,
            value: row.value,
            value_text: row.value_text,
            unit: row.unit,
            reference_low: row.reference_low,
            reference_high: row.reference_high,
            reference_text: row.reference_text,
            status: LabStatus::parse(&row.status),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LabTest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tested_on: NaiveDate,
    pub laboratory: Option<String>,
    pub notes: Option<String>,
    pub results: Vec<LabResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct LabTestRow {
    id: Uuid,
    user_id: Uuid,
    tested_on: NaiveDate,
    laboratory: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LabResultInput {
    pub biomarker_code: String,
    pub name: Option<String>,
    pub value: Option<f64>,
    pub value_text: Option<String>,
    pub unit: Option<String>,
    pub reference_low: Option<f64>,
    pub reference_high: Option<f64>,
    pub reference_text: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertLabTest {
    pub tested_on: NaiveDate,
    pub laboratory: Option<String>,
    pub notes: Option<String>,
    pub results: Vec<LabResultInput>,
}

#[utoipa::path(
    get,
    path = "/api/v1/labs/biomarkers",
    tag = "labs",
    security(("bearer_auth" = [])),
    responses((status = 200, body = [BiomarkerDefinition]))
)]
pub async fn biomarkers(
    _user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<BiomarkerDefinition>>, AppError> {
    let rows = sqlx::query_as::<_, BiomarkerDefinition>(
        "SELECT code, name, default_unit, category FROM biomarker_definitions ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn list_for(state: &AppState, user_id: Uuid) -> Result<Vec<LabTest>, AppError> {
    let tests = sqlx::query_as::<_, LabTestRow>(
        r#"
        SELECT id, user_id, tested_on, laboratory, notes, created_at, updated_at
        FROM lab_tests
        WHERE user_id = $1
        ORDER BY tested_on DESC, created_at DESC
        LIMIT 500
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;
    let mut out = Vec::with_capacity(tests.len());
    for test in tests {
        out.push(hydrate(state, test).await?);
    }
    Ok(out)
}

#[utoipa::path(
    get,
    path = "/api/v1/labs",
    tag = "labs",
    security(("bearer_auth" = [])),
    responses((status = 200, body = [LabTest]))
)]
pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<LabTest>>, AppError> {
    Ok(Json(list_for(&state, user.id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/labs",
    tag = "labs",
    security(("bearer_auth" = [])),
    request_body = UpsertLabTest,
    responses((status = 201, body = LabTest))
)]
pub async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertLabTest>,
) -> Result<(StatusCode, Json<LabTest>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(insert_panel(&state, user.id, &body).await?),
    ))
}

pub async fn insert_panel(
    state: &AppState,
    user_id: Uuid,
    body: &UpsertLabTest,
) -> Result<LabTest, AppError> {
    if body.results.is_empty() {
        return Err(AppError::validation("Add at least one biomarker result"));
    }
    let test = sqlx::query_as::<_, LabTestRow>(
        r#"
        INSERT INTO lab_tests (user_id, tested_on, laboratory, notes)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, tested_on, laboratory, notes, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(body.tested_on)
    .bind(trim_opt(body.laboratory.as_deref(), 160))
    .bind(trim_opt(body.notes.as_deref(), 2000))
    .fetch_one(&state.pool)
    .await?;
    insert_results(state, test.id, &body.results).await?;
    hydrate(state, test).await
}

#[utoipa::path(
    get,
    path = "/api/v1/labs/{id}",
    tag = "labs",
    security(("bearer_auth" = [])),
    responses((status = 200, body = LabTest))
)]
pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LabTest>, AppError> {
    let test = owned_test(&state, user.id, id).await?;
    Ok(Json(hydrate(&state, test).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/labs/{id}",
    tag = "labs",
    security(("bearer_auth" = [])),
    request_body = UpsertLabTest,
    responses((status = 200, body = LabTest))
)]
pub async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpsertLabTest>,
) -> Result<Json<LabTest>, AppError> {
    let _ = owned_test(&state, user.id, id).await?;
    if body.results.is_empty() {
        return Err(AppError::validation("Add at least one biomarker result"));
    }
    let test = sqlx::query_as::<_, LabTestRow>(
        r#"
        UPDATE lab_tests
        SET tested_on = $3, laboratory = $4, notes = $5, updated_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, tested_on, laboratory, notes, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(body.tested_on)
    .bind(trim_opt(body.laboratory.as_deref(), 160))
    .bind(trim_opt(body.notes.as_deref(), 2000))
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Lab test not found"))?;
    sqlx::query("DELETE FROM lab_results WHERE lab_test_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    insert_results(&state, id, &body.results).await?;
    Ok(Json(hydrate(&state, test).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/labs/{id}",
    tag = "labs",
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Deleted"))
)]
pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let _ = owned_test(&state, user.id, id).await?;
    sqlx::query("DELETE FROM lab_tests WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn latest_for_user(state: &AppState, user_id: Uuid) -> Result<Option<LabTest>, AppError> {
    let test = sqlx::query_as::<_, LabTestRow>(
        r#"
        SELECT id, user_id, tested_on, laboratory, notes, created_at, updated_at
        FROM lab_tests
        WHERE user_id = $1
        ORDER BY tested_on DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;
    match test {
        Some(test) => Ok(Some(hydrate(state, test).await?)),
        None => Ok(None),
    }
}

pub async fn series_for_code(
    state: &AppState,
    user_id: Uuid,
    code: &str,
    from: DateTime<Utc>,
) -> Result<Vec<(DateTime<Utc>, f64, Option<String>)>, AppError> {
    let rows = sqlx::query_as::<_, (NaiveDate, f64, Option<String>)>(
        r#"
        SELECT t.tested_on, r.value, r.unit
        FROM lab_results r
        INNER JOIN lab_tests t ON t.id = r.lab_test_id
        WHERE t.user_id = $1 AND r.biomarker_code = $2 AND r.value IS NOT NULL
          AND t.tested_on >= $3::date
        ORDER BY t.tested_on ASC
        "#,
    )
    .bind(user_id)
    .bind(code)
    .bind(from.date_naive())
    .fetch_all(&state.pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(date, value, unit)| {
            (
                date.and_hms_opt(0, 0, 0)
                    .map(|d| d.and_utc())
                    .unwrap_or(from),
                value,
                unit,
            )
        })
        .collect())
}

async fn owned_test(state: &AppState, user_id: Uuid, id: Uuid) -> Result<LabTestRow, AppError> {
    sqlx::query_as::<_, LabTestRow>(
        r#"
        SELECT id, user_id, tested_on, laboratory, notes, created_at, updated_at
        FROM lab_tests
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Lab test not found"))
}

async fn hydrate(state: &AppState, test: LabTestRow) -> Result<LabTest, AppError> {
    let rows = sqlx::query_as::<_, LabResultRow>(
        r#"
        SELECT id, lab_test_id, biomarker_code, name, value, value_text, unit,
               reference_low, reference_high, reference_text, status
        FROM lab_results
        WHERE lab_test_id = $1
        ORDER BY biomarker_code
        "#,
    )
    .bind(test.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(LabTest {
        id: test.id,
        user_id: test.user_id,
        tested_on: test.tested_on,
        laboratory: test.laboratory,
        notes: test.notes,
        results: rows.into_iter().map(LabResult::from).collect(),
        created_at: test.created_at,
        updated_at: test.updated_at,
    })
}

async fn insert_results(
    state: &AppState,
    lab_test_id: Uuid,
    results: &[LabResultInput],
) -> Result<(), AppError> {
    for result in results {
        let code = result.biomarker_code.trim().to_ascii_lowercase();
        if code.is_empty() || code.len() > 64 {
            return Err(AppError::validation("A biomarker code is required"));
        }
        if result.value.is_some_and(|value| !value.is_finite()) {
            return Err(AppError::validation("Invalid laboratory value"));
        }
        let status = derive_status(result.value, result.reference_low, result.reference_high);
        sqlx::query(
            r#"
            INSERT INTO lab_results (
                lab_test_id, biomarker_code, name, value, value_text, unit,
                reference_low, reference_high, reference_text, status
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(lab_test_id)
        .bind(&code)
        .bind(trim_opt(result.name.as_deref(), 120))
        .bind(result.value)
        .bind(trim_opt(result.value_text.as_deref(), 80))
        .bind(trim_opt(result.unit.as_deref(), 24))
        .bind(result.reference_low)
        .bind(result.reference_high)
        .bind(trim_opt(result.reference_text.as_deref(), 160))
        .bind(status.as_str())
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}

fn derive_status(value: Option<f64>, low: Option<f64>, high: Option<f64>) -> LabStatus {
    let Some(value) = value else {
        return LabStatus::Unknown;
    };
    match (low, high) {
        (Some(low), Some(high)) if high >= low => {
            if value < low {
                LabStatus::Low
            } else if value > high {
                LabStatus::High
            } else {
                LabStatus::Normal
            }
        }
        (Some(low), None) => {
            if value < low {
                LabStatus::Low
            } else {
                LabStatus::Normal
            }
        }
        (None, Some(high)) => {
            if value > high {
                LabStatus::High
            } else {
                LabStatus::Normal
            }
        }
        _ => LabStatus::Unknown,
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

    #[test]
    fn range_status_is_not_a_diagnosis() {
        assert!(matches!(
            derive_status(Some(74.0), Some(50.0), Some(125.0)),
            LabStatus::Normal
        ));
        assert!(matches!(
            derive_status(Some(10.0), Some(50.0), Some(125.0)),
            LabStatus::Low
        ));
        assert!(matches!(
            derive_status(None, Some(1.0), Some(2.0)),
            LabStatus::Unknown
        ));
    }
}
