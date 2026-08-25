use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    appointments::{self, UpsertAppointment},
    error::AppError,
    laboratory::{self, LabResultInput, UpsertLabTest},
    measurements::{self, UpsertMeasurement},
    medications::{self, UpsertMedication},
    notes::{self, UpsertNote},
    state::AppState,
    symptoms::{self, CreateSymptom},
    users::AuthUser,
    workouts::{self, UpsertWorkout, WorkoutType},
};

const MAX_MEASUREMENTS: usize = 4000;
const MAX_OTHER: usize = 500;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportPayload {
    #[serde(default)]
    pub measurements: Vec<UpsertMeasurement>,
    #[serde(default)]
    pub labs: Vec<UpsertLabTest>,
    #[serde(default)]
    pub workouts: Vec<UpsertWorkout>,
    #[serde(default)]
    pub medications: Vec<UpsertMedication>,
    #[serde(default)]
    pub symptoms: Vec<CreateSymptom>,
    #[serde(default)]
    pub appointments: Vec<UpsertAppointment>,
    #[serde(default)]
    pub notes: Vec<UpsertNote>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportResult {
    pub measurements: usize,
    pub labs: usize,
    pub workouts: usize,
    pub medications: usize,
    pub symptoms: usize,
    pub appointments: usize,
    pub notes: usize,
    pub skipped: usize,
    pub notice: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/import",
    tag = "dashboard",
    security(("bearer_auth" = [])),
    request_body = ImportPayload,
    responses((status = 201, body = ImportResult))
)]
pub async fn json(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<ImportPayload>,
) -> Result<(StatusCode, Json<ImportResult>), AppError> {
    let result = apply(&state, user.id, payload).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    post,
    path = "/api/v1/import/csv",
    tag = "dashboard",
    security(("bearer_auth" = [])),
    responses((status = 201, body = ImportResult))
)]
pub async fn csv(
    user: AuthUser,
    State(state): State<AppState>,
    body: String,
) -> Result<(StatusCode, Json<ImportResult>), AppError> {
    let body = body.trim_start_matches('\u{feff}');
    let (payload, skipped, notice) = if looks_like_healthii_csv(body) {
        let (payload, skipped) = parse_csv(body)?;
        (payload, skipped, None)
    } else {
        let parsed = crate::health_connect::parse(body)?;
        if parsed.measurements.is_empty() && parsed.workouts.is_empty() {
            return Err(AppError::validation(
                "Unrecognized CSV. Use a Healthii export or a Health Connect / Fit vitals CSV.",
            ));
        }
        (
            ImportPayload {
                measurements: parsed.measurements,
                labs: Vec::new(),
                workouts: parsed.workouts,
                medications: Vec::new(),
                symptoms: Vec::new(),
                appointments: Vec::new(),
                notes: Vec::new(),
            },
            parsed.skipped,
            Some(
                "Imported Health Connect / Fit CSV into your account. Steps and dense heart-rate samples are skipped.",
            ),
        )
    };
    let mut result = apply(&state, user.id, payload).await?;
    result.skipped += skipped;
    if let Some(notice) = notice {
        result.notice = notice.into();
    }
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    post,
    path = "/api/v1/import/apple-health",
    tag = "dashboard",
    security(("bearer_auth" = [])),
    responses((status = 201, body = ImportResult))
)]
pub async fn apple_health(
    user: AuthUser,
    State(state): State<AppState>,
    body: String,
) -> Result<(StatusCode, Json<ImportResult>), AppError> {
    let parsed = crate::apple_health::parse(&body)?;
    let skipped = parsed.skipped;
    let payload = ImportPayload {
        measurements: parsed.measurements,
        labs: Vec::new(),
        workouts: parsed.workouts,
        medications: Vec::new(),
        symptoms: Vec::new(),
        appointments: Vec::new(),
        notes: Vec::new(),
    };
    let mut result = apply(&state, user.id, payload).await?;
    result.skipped += skipped;
    result.notice =
        "Imported Apple Health records into your account. Intraday heart rate and steps are skipped."
            .into();
    Ok((StatusCode::CREATED, Json(result)))
}

async fn apply(
    state: &AppState,
    user_id: Uuid,
    payload: ImportPayload,
) -> Result<ImportResult, AppError> {
    if payload.measurements.len() > MAX_MEASUREMENTS
        || payload.labs.len() > MAX_OTHER
        || payload.workouts.len() > MAX_OTHER
        || payload.medications.len() > MAX_OTHER
        || payload.symptoms.len() > MAX_OTHER
        || payload.appointments.len() > MAX_OTHER
        || payload.notes.len() > MAX_OTHER
    {
        return Err(AppError::validation(
            "Import is larger than the allowed batch",
        ));
    }

    let mut measurements = 0usize;
    for mut row in payload.measurements {
        let source = row.source.as_deref().map(str::trim).unwrap_or("");
        if source.is_empty() {
            row.source = Some("import".into());
        }
        measurements::insert_measurement(state, user_id, &row).await?;
        measurements += 1;
    }

    let mut labs = 0usize;
    for row in payload.labs {
        laboratory::insert_panel(state, user_id, &row).await?;
        labs += 1;
    }

    let mut workouts = 0usize;
    for row in payload.workouts {
        workouts::insert(state, user_id, &row).await?;
        workouts += 1;
    }

    let mut medications = 0usize;
    for row in payload.medications {
        medications::insert(state, user_id, &row).await?;
        medications += 1;
    }

    let mut symptoms = 0usize;
    for row in payload.symptoms {
        symptoms::insert(state, user_id, &row).await?;
        symptoms += 1;
    }

    let mut appointments = 0usize;
    for row in payload.appointments {
        appointments::insert(state, user_id, &row).await?;
        appointments += 1;
    }

    let mut notes = 0usize;
    for row in payload.notes {
        notes::insert(state, user_id, &row).await?;
        notes += 1;
    }

    crate::audit::record(state, user_id, "import.records", Some("import")).await;

    Ok(ImportResult {
        measurements,
        labs,
        workouts,
        medications,
        symptoms,
        appointments,
        notes,
        skipped: 0,
        notice: "Imported into your account. Document files are not part of JSON/CSV export."
            .into(),
    })
}

fn looks_like_healthii_csv(body: &str) -> bool {
    let Some(line) = body.lines().find(|line| !line.trim().is_empty()) else {
        return true;
    };
    let fields = split_csv(line);
    let head = fields
        .first()
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    matches!(
        head.as_str(),
        "entity"
            | "measurement"
            | "lab_result"
            | "workout"
            | "medication"
            | "symptom"
            | "appointment"
            | "note"
    )
}

fn parse_csv(body: &str) -> Result<(ImportPayload, usize), AppError> {
    let mut payload = ImportPayload {
        measurements: Vec::new(),
        labs: Vec::new(),
        workouts: Vec::new(),
        medications: Vec::new(),
        symptoms: Vec::new(),
        appointments: Vec::new(),
        notes: Vec::new(),
    };
    let mut skipped = 0usize;
    let mut lab_groups: Vec<(NaiveDate, Vec<LabResultInput>)> = Vec::new();

    for (index, line) in body.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields = split_csv(line);
        if index == 0
            && fields
                .first()
                .is_some_and(|value| value.eq_ignore_ascii_case("entity"))
        {
            continue;
        }
        if fields.len() < 7 {
            skipped += 1;
            continue;
        }
        let entity = fields[0].trim().to_ascii_lowercase();
        match entity.as_str() {
            "measurement" => match parse_measurement(&fields) {
                Ok(row) => payload.measurements.push(row),
                Err(_) => skipped += 1,
            },
            "lab_result" => match parse_lab_result(&fields) {
                Ok((date, result)) => {
                    if let Some(group) = lab_groups
                        .iter_mut()
                        .find(|(existing, _)| *existing == date)
                    {
                        group.1.push(result);
                    } else {
                        lab_groups.push((date, vec![result]));
                    }
                }
                Err(_) => skipped += 1,
            },
            "workout" => match parse_workout(&fields) {
                Ok(row) => payload.workouts.push(row),
                Err(_) => skipped += 1,
            },
            "medication" => {
                let name = fields[6].trim();
                if name.is_empty() {
                    skipped += 1;
                    continue;
                }
                payload.medications.push(UpsertMedication {
                    name: name.into(),
                    kind: Some(if fields[3].trim().is_empty() {
                        "medication".into()
                    } else {
                        fields[3].trim().into()
                    }),
                    dosage: none_if_empty(&fields[4]),
                    schedule: None,
                    started_on: None,
                    ended_on: None,
                    notes: none_if_empty(&fields[5]),
                });
            }
            "symptom" => {
                let name = fields[6].trim();
                if name.is_empty() {
                    skipped += 1;
                    continue;
                }
                payload.symptoms.push(CreateSymptom {
                    name: name.into(),
                    severity: fields[4].trim().parse().ok(),
                    noted_at: parse_time(&fields[2]).ok(),
                    notes: none_if_empty(&fields[5]),
                });
            }
            "appointment" => {
                let title = fields[6].trim();
                let Ok(starts) = parse_time(&fields[2]) else {
                    skipped += 1;
                    continue;
                };
                if title.is_empty() {
                    skipped += 1;
                    continue;
                }
                payload.appointments.push(UpsertAppointment {
                    title: title.into(),
                    location: None,
                    provider: none_if_empty(&fields[3]),
                    starts_at: starts,
                    ends_at: None,
                    notes: none_if_empty(&fields[5]),
                });
            }
            "note" => {
                let title = fields[6].trim();
                if title.is_empty() {
                    skipped += 1;
                    continue;
                }
                payload.notes.push(UpsertNote {
                    title: title.into(),
                    body: none_if_empty(&fields[5]),
                    noted_at: parse_time(&fields[2]).ok(),
                });
            }
            _ => skipped += 1,
        }
    }

    for (tested_on, results) in lab_groups {
        if results.is_empty() {
            continue;
        }
        payload.labs.push(UpsertLabTest {
            tested_on,
            laboratory: None,
            notes: Some("Imported".into()),
            results,
        });
    }

    Ok((payload, skipped))
}

fn parse_measurement(fields: &[String]) -> Result<UpsertMeasurement, AppError> {
    let value = fields[4]
        .trim()
        .parse::<f64>()
        .map_err(|_| AppError::validation("Invalid measurement value"))?;
    Ok(UpsertMeasurement {
        kind: fields[3].trim().into(),
        value,
        unit: fields[5].trim().into(),
        measured_at: parse_time(&fields[2]).ok(),
        source: Some("import".into()),
        notes: none_if_empty(&fields[6]),
    })
}

fn parse_lab_result(fields: &[String]) -> Result<(NaiveDate, LabResultInput), AppError> {
    let date = parse_date(&fields[2])?;
    let value = fields[4].trim().parse::<f64>().ok();
    Ok((
        date,
        LabResultInput {
            biomarker_code: fields[3].trim().into(),
            name: none_if_empty(&fields[6]),
            value,
            value_text: None,
            unit: none_if_empty(&fields[5]),
            reference_low: None,
            reference_high: None,
            reference_text: None,
        },
    ))
}

fn parse_workout(fields: &[String]) -> Result<UpsertWorkout, AppError> {
    Ok(UpsertWorkout {
        workout_type: WorkoutType::parse(fields[3].trim())?,
        started_at: parse_time(&fields[2]).ok(),
        duration_seconds: None,
        distance: None,
        distance_unit: None,
        calories: None,
        notes: none_if_empty(&fields[6]),
        exercises: Vec::new(),
    })
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, AppError> {
    let value = value.trim();
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()
                .and_then(|date| date.and_hms_opt(8, 0, 0))
                .map(|naive| naive.and_utc())
                .ok_or_else(|| AppError::validation("Invalid timestamp"))
        })
}

fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    let value = value.trim();
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(date);
    }
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.date_naive())
        .map_err(|_| AppError::validation("Invalid date"))
}

fn none_if_empty(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.into())
    }
}

fn split_csv(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    quoted = false;
                }
            }
            '"' => quoted = true,
            ',' if !quoted => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_healthii_csv_export_shape() {
        let csv = "entity,id,occurred_at,type,value,unit,title\nmeasurement,,2026-08-01T00:00:00Z,weight,80.5,kg,\nlab_result,,2026-01-15,vitamin_d,70,nmol/L,Vitamin D\nnote,,2026-08-01T00:00:00Z,note,,Ask about labs,Clinic\n";
        let (payload, skipped) = parse_csv(csv).unwrap();
        assert_eq!(skipped, 0);
        assert_eq!(payload.measurements.len(), 1);
        assert_eq!(payload.measurements[0].kind, "weight");
        assert_eq!(payload.labs.len(), 1);
        assert_eq!(payload.labs[0].results[0].biomarker_code, "vitamin_d");
        assert_eq!(payload.notes.len(), 1);
        assert_eq!(payload.notes[0].title, "Clinic");
    }

    #[test]
    fn skips_unknown_csv_entities() {
        let csv = "mystery,,2026-08-01,x,1,u,\n";
        let (payload, skipped) = parse_csv(csv).unwrap();
        assert_eq!(skipped, 1);
        assert!(payload.measurements.is_empty());
    }
}
