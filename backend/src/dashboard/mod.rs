use axum::Json;
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    appointments, documents, error::AppError, laboratory, measurements, medications, notes,
    profile, state::AppState, symptoms, users::AuthUser, workouts,
};

const DISCLAIMER: &str = "Healthii is a personal health tracking and organization tool. It does not provide medical diagnosis or replace professional medical advice.";

#[derive(Serialize, ToSchema)]
pub struct DashboardResponse {
    pub mode: String,
    pub disclaimer: String,
    pub generated_at: chrono::DateTime<Utc>,
    pub widgets: Vec<DashboardWidget>,
}

#[derive(Serialize, ToSchema)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub empty: bool,
    pub summary: Option<String>,
    pub hint: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard",
    tag = "dashboard",
    security(("bearer_auth" = [])),
    responses((status = 200, body = DashboardResponse))
)]
pub async fn get_dashboard(
    user: AuthUser,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<DashboardResponse>, AppError> {
    let weight = measurements::latest(&state, user.id, "weight").await?;
    let systolic = measurements::latest(&state, user.id, "blood_pressure_systolic").await?;
    let diastolic = measurements::latest(&state, user.id, "blood_pressure_diastolic").await?;
    let heart = measurements::latest(&state, user.id, "resting_heart_rate").await?;
    let sleep = measurements::latest(&state, user.id, "sleep").await?;
    let recent = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM measurements WHERE user_id = $1 AND measured_at >= now() - interval '30 days'",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let lab = laboratory::latest_for_user(&state, user.id).await?;
    let workouts = workouts::recent(&state, user.id, 3).await?;
    let appointments = appointments::upcoming(&state, user.id, 3).await?;
    let documents = documents::recent(&state, user.id, 3).await?;
    let active_meds = medications::active(&state, user.id, 5).await?;
    let latest_note = notes::latest(&state, user.id).await?;
    let recent_symptoms = symptoms::recent(&state, user.id, 3).await?;
    let profile = profile::load(&state, user.id).await?;

    let trend_7 = measurements::since(&state, user.id, "weight", measurements::days_ago(7)).await?;
    let trend_30 =
        measurements::since(&state, user.id, "weight", measurements::days_ago(30)).await?;
    let trend_90 =
        measurements::since(&state, user.id, "weight", measurements::days_ago(90)).await?;

    let weight_summary = weight
        .as_ref()
        .map(|row| format!("{:.1} {}", row.value, row.unit));
    let mut trend_bits = Vec::new();
    if let Some(delta) = measurements::trend_delta(&trend_7) {
        trend_bits.push(format!("7d {delta:+.1}"));
    }
    if let Some(delta) = measurements::trend_delta(&trend_30) {
        trend_bits.push(format!("30d {delta:+.1}"));
    }
    if let Some(delta) = measurements::trend_delta(&trend_90) {
        trend_bits.push(format!("90d {delta:+.1}"));
    }

    let bmi_hint = match (profile.height_cm, weight.as_ref()) {
        (Some(height_cm), Some(weight)) if height_cm > 0.0 => {
            let kg = if weight.unit.eq_ignore_ascii_case("lb") {
                weight.value * 0.45359237
            } else {
                weight.value
            };
            let meters = height_cm / 100.0;
            let bmi = kg / (meters * meters);
            Some(format!(
                "Derived BMI {:.1} from stored height and latest weight — not a diagnosis",
                bmi
            ))
        }
        _ => None,
    };

    Ok(Json(DashboardResponse {
        mode: "live".into(),
        disclaimer: DISCLAIMER.into(),
        generated_at: Utc::now(),
        widgets: vec![
            widget(
                "weight",
                "Current weight",
                "metric",
                weight_summary.clone(),
                Some("Original unit preserved".into()),
            ),
            widget(
                "weight-goal",
                "Weight goal",
                "metric",
                match (profile.goal_weight, weight.as_ref()) {
                    (Some(goal), Some(current)) => {
                        let goal_unit = profile
                            .goal_weight_unit
                            .as_deref()
                            .unwrap_or(current.unit.as_str());
                        if goal_unit == current.unit {
                            Some(format!(
                                "{:.1} {} · {:+.1} to goal",
                                current.value,
                                current.unit,
                                current.value - goal
                            ))
                        } else {
                            Some(format!(
                                "Goal {:.1} {} · now {:.1} {}",
                                goal, goal_unit, current.value, current.unit
                            ))
                        }
                    }
                    (Some(goal), None) => Some(format!(
                        "{:.1} {}",
                        goal,
                        profile.goal_weight_unit.as_deref().unwrap_or("kg")
                    )),
                    _ => None,
                },
                Some("A personal target, not a medical recommendation".into()),
            ),
            widget(
                "weight-trend",
                "Weight trend",
                "chart",
                if trend_bits.is_empty() {
                    None
                } else {
                    Some(trend_bits.join(" · "))
                },
                bmi_hint,
            ),
            widget(
                "labs",
                "Latest blood tests",
                "list",
                lab.as_ref()
                    .map(|test| format!("{} · {} biomarkers", test.tested_on, test.results.len())),
                Some("Flags compare a value to that report's range only".into()),
            ),
            widget(
                "measurements",
                "Recent measurements",
                "list",
                Some(format!("{recent} in the last 30 days")),
                None,
            ),
            widget(
                "blood-pressure",
                "Latest blood pressure",
                "metric",
                match (systolic, diastolic) {
                    (Some(sys), Some(dia)) => {
                        Some(format!("{:.0} / {:.0} {}", sys.value, dia.value, sys.unit))
                    }
                    _ => None,
                },
                None,
            ),
            widget(
                "heart-rate",
                "Resting heart rate",
                "metric",
                heart.map(|row| format!("{:.0} {}", row.value, row.unit)),
                None,
            ),
            widget(
                "sleep",
                "Sleep",
                "metric",
                sleep.map(|row| format!("{:.1} {}", row.value, row.unit)),
                Some("Hours slept — not a sleep study".into()),
            ),
            widget(
                "workouts",
                "Recent workouts",
                "list",
                if workouts.is_empty() {
                    None
                } else {
                    Some(format!("{} logged", workouts.len()))
                },
                None,
            ),
            widget(
                "medications",
                "Medications",
                "list",
                if active_meds.is_empty() {
                    None
                } else {
                    Some(
                        active_meds
                            .iter()
                            .take(3)
                            .map(|row| row.name.as_str())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                },
                active_meds
                    .first()
                    .and_then(|row| row.schedule.clone())
                    .or_else(|| {
                        if active_meds.is_empty() {
                            None
                        } else {
                            Some("Names only — not a dosing instruction".into())
                        }
                    }),
            ),
            widget(
                "appointments",
                "Upcoming appointments",
                "list",
                appointments.first().map(appointment_when),
                None,
            ),
            widget(
                "symptoms",
                "Recent symptoms",
                "list",
                if recent_symptoms.is_empty() {
                    None
                } else {
                    Some(
                        recent_symptoms
                            .iter()
                            .map(|row| row.name.as_str())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                },
                Some("How you felt — not a diagnosis".into()),
            ),
            widget(
                "notes",
                "Latest note",
                "list",
                latest_note.as_ref().map(|note| note.title.clone()),
                latest_note
                    .as_ref()
                    .map(|note| note.body.chars().take(80).collect::<String>()),
            ),
            widget(
                "documents",
                "Recent documents",
                "list",
                if documents.is_empty() {
                    None
                } else {
                    Some(format!("{} files", documents.len()))
                },
                None,
            ),
            widget(
                "timeline",
                "Health timeline",
                "timeline",
                Some("Open Timeline for the full chronology".into()),
                None,
            ),
        ],
    }))
}

fn appointment_when(row: &appointments::Appointment) -> String {
    let days = (row.starts_at.date_naive() - Utc::now().date_naive()).num_days();
    match days {
        0 => format!("Today · {}", row.title),
        1 => format!("Tomorrow · {}", row.title),
        d if d > 1 => format!("In {d} days · {}", row.title),
        _ => row.title.clone(),
    }
}

fn widget(
    id: &str,
    title: &str,
    kind: &str,
    summary: Option<String>,
    hint: Option<String>,
) -> DashboardWidget {
    DashboardWidget {
        id: id.into(),
        title: title.into(),
        kind: kind.into(),
        empty: summary.is_none(),
        summary,
        hint,
    }
}
