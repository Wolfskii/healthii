use axum::{extract::Query, extract::State, Json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppError, laboratory, measurements, state::AppState, users::AuthUser};

#[derive(Debug, Deserialize)]
pub struct ChartQuery {
    pub metric: String,
    pub days: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChartPoint {
    pub t: DateTime<Utc>,
    pub v: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChartResponse {
    pub metric: String,
    pub unit: Option<String>,
    pub source: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub average: Option<f64>,
    pub points: Vec<ChartPoint>,
    pub notice: String,
}

pub async fn get(
    user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ChartQuery>,
) -> Result<Json<ChartResponse>, AppError> {
    let days = query.days.unwrap_or(90).clamp(7, 3650);
    let from = Utc::now() - Duration::days(days);
    let metric = query.metric.trim().to_ascii_lowercase();
    let notice = "These figures are recorded values, not a diagnosis.".to_string();

    if measurements::MEASUREMENT_TYPES.contains(&metric.as_str()) {
        let series = measurements::since(&state, user.id, &metric, from).await?;
        let unit = series.first().map(|row| row.unit.clone());
        let values: Vec<f64> = series.iter().map(|row| row.value).collect();
        return Ok(Json(ChartResponse {
            metric,
            unit,
            source: "recorded data".into(),
            min: values.iter().cloned().reduce(f64::min),
            max: values.iter().cloned().reduce(f64::max),
            average: mean(&values),
            points: series
                .into_iter()
                .map(|row| ChartPoint {
                    t: row.measured_at,
                    v: row.value,
                })
                .collect(),
            notice,
        }));
    }

    let series = laboratory::series_for_code(&state, user.id, &metric, from).await?;
    let unit = series.iter().find_map(|(_, _, unit)| unit.clone());
    let values: Vec<f64> = series.iter().map(|(_, value, _)| *value).collect();
    Ok(Json(ChartResponse {
        metric,
        unit,
        source: "laboratory reference range comparison is separate from this chart".into(),
        min: values.iter().cloned().reduce(f64::min),
        max: values.iter().cloned().reduce(f64::max),
        average: mean(&values),
        points: series
            .into_iter()
            .map(|(t, v, _)| ChartPoint { t, v })
            .collect(),
        notice,
    }))
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}
