use axum::{
    extract::{Query, State},
    http::{header, HeaderValue},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    appointments, documents, error::AppError, laboratory, medications, notes, profile,
    state::AppState, symptoms, users::AuthUser, workouts,
};

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExportPayload {
    pub exported_at: String,
    pub disclaimer: String,
    pub measurements: Vec<crate::measurements::Measurement>,
    pub labs: Vec<laboratory::LabTest>,
    pub workouts: Vec<workouts::Workout>,
    pub medications: Vec<medications::Medication>,
    pub symptoms: Vec<symptoms::Symptom>,
    pub appointments: Vec<appointments::Appointment>,
    pub notes: Vec<notes::Note>,
    pub documents: Vec<documents::Document>,
    pub profile: profile::Profile,
}

pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let payload = load(&state, user.id).await?;
    if query.format.as_deref() == Some("csv") {
        let mut response = to_csv(&payload).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/csv; charset=utf-8"),
        );
        response.headers_mut().insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment; filename=\"healthii-export.csv\""),
        );
        return Ok(response);
    }
    Ok(Json(payload).into_response())
}

async fn load(state: &AppState, user_id: uuid::Uuid) -> Result<ExportPayload, AppError> {
    let measurement_rows = sqlx::query_as::<_, crate::measurements::Measurement>(
        r#"
        SELECT id, user_id, type, value, unit, measured_at, source, notes, created_at, updated_at
        FROM measurements WHERE user_id = $1 ORDER BY measured_at
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(ExportPayload {
        exported_at: Utc::now().to_rfc3339(),
        disclaimer: "Healthii export of your own records. Not a medical diagnosis.".into(),
        measurements: measurement_rows,
        labs: laboratory::list_for(state, user_id).await?,
        workouts: workouts::recent(state, user_id, 500).await?,
        medications: sqlx::query_as(
            r#"
            SELECT id, user_id, name, kind, dosage, schedule, started_on, ended_on, notes, created_at, updated_at
            FROM medications WHERE user_id = $1 ORDER BY name
            "#,
        )
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?,
        symptoms: sqlx::query_as(
            r#"
            SELECT id, user_id, name, severity, noted_at, notes, created_at
            FROM symptoms WHERE user_id = $1 ORDER BY noted_at
            "#,
        )
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?,
        appointments: sqlx::query_as(
            r#"
            SELECT id, user_id, title, location, provider, starts_at, ends_at, notes, created_at
            FROM appointments WHERE user_id = $1 ORDER BY starts_at
            "#,
        )
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?,
        notes: notes::list_for(state, user_id).await?,
        documents: documents::recent(state, user_id, 500).await?,
        profile: profile::load(state, user_id).await?,
    })
}

fn to_csv(payload: &ExportPayload) -> String {
    let mut out = String::from("entity,id,occurred_at,type,value,unit,title\n");
    for row in &payload.measurements {
        write_csv_row(
            &mut out,
            [
                "measurement",
                &row.id.to_string(),
                &row.measured_at.to_rfc3339(),
                &row.kind,
                &row.value.to_string(),
                &row.unit,
                "",
            ],
        );
    }
    for lab in &payload.labs {
        for result in &lab.results {
            write_csv_row(
                &mut out,
                [
                    "lab_result",
                    &result.id.to_string(),
                    &lab.tested_on.to_string(),
                    &result.biomarker_code,
                    &result
                        .value
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    result.unit.as_deref().unwrap_or(""),
                    result.name.as_deref().unwrap_or(""),
                ],
            );
        }
    }
    for row in &payload.workouts {
        write_csv_row(
            &mut out,
            [
                "workout",
                &row.id.to_string(),
                &row.started_at.to_rfc3339(),
                row.workout_type.as_str(),
                "",
                "",
                row.notes.as_deref().unwrap_or(""),
            ],
        );
    }
    for row in &payload.medications {
        write_csv_row(
            &mut out,
            [
                "medication",
                &row.id.to_string(),
                "",
                &row.kind,
                row.dosage.as_deref().unwrap_or(""),
                row.notes.as_deref().unwrap_or(""),
                &row.name,
            ],
        );
    }
    for row in &payload.symptoms {
        write_csv_row(
            &mut out,
            [
                "symptom",
                &row.id.to_string(),
                &row.noted_at.to_rfc3339(),
                "",
                &row.severity
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.notes.as_deref().unwrap_or(""),
                &row.name,
            ],
        );
    }
    for row in &payload.appointments {
        write_csv_row(
            &mut out,
            [
                "appointment",
                &row.id.to_string(),
                &row.starts_at.to_rfc3339(),
                row.provider.as_deref().unwrap_or(""),
                "",
                row.notes.as_deref().unwrap_or(""),
                &row.title,
            ],
        );
    }
    for row in &payload.notes {
        write_csv_row(
            &mut out,
            [
                "note",
                &row.id.to_string(),
                &row.noted_at.to_rfc3339(),
                "note",
                "",
                &row.body,
                &row.title,
            ],
        );
    }
    out
}

fn write_csv_row(out: &mut String, fields: [&str; 7]) {
    out.push_str(
        &fields
            .iter()
            .map(|field| escape(field))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');
}

fn escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
