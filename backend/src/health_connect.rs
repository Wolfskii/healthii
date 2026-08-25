use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use crate::{error::AppError, measurements::UpsertMeasurement, workouts::UpsertWorkout};

pub struct HealthConnectImport {
    pub measurements: Vec<UpsertMeasurement>,
    pub workouts: Vec<UpsertWorkout>,
    pub skipped: usize,
}

const KEEP_MEASUREMENTS: usize = 4000;

pub fn parse(csv: &str) -> Result<HealthConnectImport, AppError> {
    let csv = csv.trim_start_matches('\u{feff}');
    let mut lines = csv.lines().filter(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    });
    let Some(header_line) = lines.next() else {
        return Err(AppError::validation("CSV file is empty"));
    };
    let headers = split_csv(header_line);
    if headers.len() < 2 {
        return Err(AppError::validation(
            "Unrecognized CSV. Use a Healthii export or a Health Connect / Fit vitals CSV.",
        ));
    }

    let time_index = headers.iter().enumerate().find_map(|(index, header)| {
        matches!(
            normalize_header(header).as_str(),
            "start time"
                | "start"
                | "date"
                | "time"
                | "datetime"
                | "timestamp"
                | "date time"
                | "record time"
                | "start date"
                | "measured at"
        )
        .then_some(index)
    });

    let columns: Vec<(usize, String, String)> = headers
        .iter()
        .enumerate()
        .filter_map(|(index, header)| {
            if time_index == Some(index) {
                return None;
            }
            let (kind, unit) = map_column(header)?;
            Some((index, kind, unit))
        })
        .collect();

    if columns.is_empty() {
        return Err(AppError::validation(
            "No supported Health Connect columns found (weight, blood pressure, resting heart rate, glucose, …)",
        ));
    }

    let mut measurements = Vec::new();
    let mut skipped = 0usize;
    for line in lines {
        let fields = split_csv(line);
        let measured_at = time_index
            .and_then(|index| fields.get(index))
            .and_then(|value| parse_flexible_time(value).ok());
        for (index, kind, unit) in &columns {
            let Some(raw) = fields.get(*index).map(|value| value.trim()) else {
                skipped += 1;
                continue;
            };
            if raw.is_empty() {
                continue;
            }
            let Ok(raw_value) = raw.parse::<f64>() else {
                skipped += 1;
                continue;
            };
            if !raw_value.is_finite() {
                skipped += 1;
                continue;
            }
            let (value, unit) = scale_value(kind, raw_value, unit.clone());
            measurements.push(UpsertMeasurement {
                kind: kind.clone(),
                value,
                unit,
                measured_at,
                source: Some("health_connect".into()),
                notes: None,
            });
        }
    }

    if measurements.len() > KEEP_MEASUREMENTS {
        skipped += measurements.len() - KEEP_MEASUREMENTS;
        measurements.truncate(KEEP_MEASUREMENTS);
    }

    Ok(HealthConnectImport {
        measurements,
        workouts: Vec::new(),
        skipped,
    })
}

fn map_column(header: &str) -> Option<(String, String)> {
    let unit = unit_from_header(header);
    let name = normalize_header(header);
    if name.contains("step")
        || name.contains("calorie")
        || name.contains("distance")
        || name.contains("floor")
        || name.contains("sleep stage")
        || name.contains("sleep analysis")
        || name.contains("deep sleep")
        || name.contains("rem sleep")
        || name.contains("light sleep")
        || name.contains("awake")
        || name.contains("vo2")
        || name.contains("hrv")
        || name.contains("variability")
        || name.contains("zone")
        || name == "heart rate"
        || name == "bpm"
    {
        return None;
    }
    let kind = match name.as_str() {
        "weight" | "body mass" | "body weight" | "weight kg" | "weight lb" => "weight",
        "height" | "body height" => "height",
        "bmi" | "body mass index" => "bmi",
        "body fat" | "body fat percentage" | "body fat percent" => "body_fat",
        "resting heart rate" | "resting hr" | "rhr" => "resting_heart_rate",
        "systolic" | "systolic blood pressure" | "blood pressure systolic" => {
            "blood_pressure_systolic"
        }
        "diastolic" | "diastolic blood pressure" | "blood pressure diastolic" => {
            "blood_pressure_diastolic"
        }
        "blood glucose" | "glucose" | "blood sugar" => "blood_glucose",
        "body temperature" | "temperature" => "body_temperature",
        "oxygen saturation" | "spo2" | "spo 2" => "oxygen_saturation",
        "waist" | "waist circumference" => "waist_circumference",
        "sleep"
        | "sleep hours"
        | "sleep duration"
        | "sleep duration hours"
        | "time in bed"
        | "time asleep"
        | "hours asleep" => "sleep",
        _ => return None,
    };
    let unit = unit.unwrap_or_else(|| default_unit(kind).into());
    Some((kind.into(), unit))
}

fn default_unit(kind: &str) -> &'static str {
    match kind {
        "weight" => "kg",
        "height" => "cm",
        "bmi" => "kg/m²",
        "body_fat" | "oxygen_saturation" => "%",
        "blood_pressure_systolic" | "blood_pressure_diastolic" => "mmHg",
        "resting_heart_rate" => "bpm",
        "blood_glucose" => "mg/dL",
        "body_temperature" => "°C",
        "waist_circumference" => "cm",
        "sleep" => "h",
        _ => "unit",
    }
}

fn scale_value(kind: &str, value: f64, mut unit: String) -> (f64, String) {
    let mut value = value;
    if kind == "height" && unit == "m" && value < 3.5 {
        value *= 100.0;
        unit = "cm".into();
    }
    if kind == "oxygen_saturation" && value <= 1.0 {
        value *= 100.0;
        unit = "%".into();
    }
    if kind == "body_fat" && value <= 1.0 && !unit.contains('%') {
        value *= 100.0;
        unit = "%".into();
    }
    if kind == "sleep" {
        (value, unit) = crate::measurements::scale_sleep_import(value, &unit);
    }
    (value, unit)
}

fn unit_from_header(header: &str) -> Option<String> {
    let start = header.find('(')?;
    let end = header.find(')')?;
    if end <= start + 1 {
        return None;
    }
    let unit = header[start + 1..end].trim();
    if unit.is_empty() {
        return None;
    }
    Some(normalize_unit(unit))
}

fn normalize_unit(unit: &str) -> String {
    match unit.trim() {
        "count/min" | "beats/min" | "beats per minute" => "bpm".into(),
        "lb" | "lbs" | "pound" | "pounds" => "lb".into(),
        "degC" | "°C" | "C" | "celsius" => "°C".into(),
        "degF" | "°F" | "F" | "fahrenheit" => "°F".into(),
        "kg" | "cm" | "m" | "%" | "mmHg" | "mmol/L" | "mg/dL" | "bpm" => unit.trim().into(),
        other => other.chars().take(24).collect(),
    }
}

fn normalize_header(header: &str) -> String {
    let lower = header.trim().to_ascii_lowercase();
    let without_unit = lower
        .split_once('(')
        .map(|(head, _)| head)
        .unwrap_or(&lower)
        .trim();
    without_unit
        .replace(['_', '-', '/'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_flexible_time(value: &str) -> Result<DateTime<Utc>, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::validation("Empty timestamp"));
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }
    if let Ok(parsed) = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S %z") {
        return Ok(parsed.with_timezone(&Utc));
    }
    const NAIVE: &[&str] = &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
    ];
    for format in NAIVE {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(naive.and_utc());
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date
            .and_hms_opt(8, 0, 0)
            .map(|naive| naive.and_utc())
            .ok_or_else(|| AppError::validation("Invalid timestamp"));
    }
    Err(AppError::validation("Invalid timestamp"))
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
            ',' if !quoted => fields.push(std::mem::take(&mut current)),
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
    fn parses_wide_vitals_csv_and_skips_steps() {
        let csv = "Start time,Weight (kg),Resting heart rate (bpm),Steps,Systolic blood pressure (mmHg),Diastolic blood pressure (mmHg)\n2026-01-15 07:30:00,79.1,58,4321,118,76\n";
        let parsed = parse(csv).unwrap();
        assert_eq!(parsed.measurements.len(), 4);
        assert!(parsed
            .measurements
            .iter()
            .any(|row| row.kind == "weight" && (row.value - 79.1).abs() < f64::EPSILON));
        assert!(parsed
            .measurements
            .iter()
            .all(|row| row.source.as_deref() == Some("health_connect")));
        assert_eq!(parsed.skipped, 0);
    }

    #[test]
    fn parses_sleep_duration_minutes() {
        let csv = "Start time,Sleep duration (min)\n2026-01-15 22:00:00,450\n";
        let parsed = parse(csv).unwrap();
        assert_eq!(parsed.measurements.len(), 1);
        assert_eq!(parsed.measurements[0].kind, "sleep");
        assert!((parsed.measurements[0].value - 7.5).abs() < f64::EPSILON);
        assert_eq!(parsed.measurements[0].unit, "h");
    }

    #[test]
    fn rejects_unknown_headers() {
        assert!(parse("foo,bar\n1,2\n").is_err());
    }
}
