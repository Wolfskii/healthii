use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::{
    error::AppError,
    measurements::UpsertMeasurement,
    workouts::{UpsertWorkout, WorkoutType},
};

pub struct AppleImport {
    pub measurements: Vec<UpsertMeasurement>,
    pub workouts: Vec<UpsertWorkout>,
    pub skipped: usize,
}

const KEEP_MEASUREMENTS: usize = 4000;

pub fn parse(xml: &str) -> Result<AppleImport, AppError> {
    if !xml.contains("<HealthData") && !xml.contains("<Record ") {
        return Err(AppError::validation(
            "This file does not look like an Apple Health export.xml",
        ));
    }

    let mut measurements = Vec::new();
    let mut skipped = 0usize;

    for attrs in tags(xml, "Record") {
        match record_to_measurement(&attrs) {
            Ok(Some(row)) => measurements.push(row),
            Ok(None) => skipped += 1,
            Err(_) => skipped += 1,
        }
    }

    let mut workouts = Vec::new();
    for attrs in tags(xml, "Workout") {
        match workout_from_attrs(&attrs) {
            Ok(Some(row)) => workouts.push(row),
            Ok(None) => skipped += 1,
            Err(_) => skipped += 1,
        }
    }

    if measurements.len() > KEEP_MEASUREMENTS {
        skipped += measurements.len() - KEEP_MEASUREMENTS;
        measurements.sort_by(|a, b| {
            let rank = |kind: &str| match kind {
                "weight"
                | "blood_pressure_systolic"
                | "blood_pressure_diastolic"
                | "resting_heart_rate"
                | "blood_glucose"
                | "height"
                | "sleep" => 0,
                _ => 1,
            };
            rank(&a.kind).cmp(&rank(&b.kind))
        });
        measurements.truncate(KEEP_MEASUREMENTS);
    }

    if workouts.len() > 500 {
        skipped += workouts.len() - 500;
        workouts.truncate(500);
    }

    Ok(AppleImport {
        measurements,
        workouts,
        skipped,
    })
}

fn record_to_measurement(
    attrs: &HashMap<String, String>,
) -> Result<Option<UpsertMeasurement>, AppError> {
    let hk_type = attrs.get("type").map(String::as_str).unwrap_or("");
    if let Some((kind, value, unit)) = sleep_from_attrs(hk_type, attrs) {
        let measured_at = attrs
            .get("startDate")
            .or_else(|| attrs.get("endDate"))
            .and_then(|value| parse_apple_time(value).ok());
        return Ok(Some(UpsertMeasurement {
            kind,
            value,
            unit,
            measured_at,
            source: Some("apple_health".into()),
            notes: attrs.get("sourceName").cloned(),
        }));
    }
    let Some((kind, mut value, mut unit)) = map_quantity(hk_type, attrs)? else {
        return Ok(None);
    };
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
    let measured_at = attrs
        .get("startDate")
        .or_else(|| attrs.get("endDate"))
        .and_then(|value| parse_apple_time(value).ok());
    Ok(Some(UpsertMeasurement {
        kind,
        value,
        unit,
        measured_at,
        source: Some("apple_health".into()),
        notes: attrs.get("sourceName").cloned(),
    }))
}

fn map_quantity(
    hk_type: &str,
    attrs: &HashMap<String, String>,
) -> Result<Option<(String, f64, String)>, AppError> {
    let value = match attrs
        .get("value")
        .and_then(|value| value.parse::<f64>().ok())
    {
        Some(value) if value.is_finite() => value,
        _ => return Ok(None),
    };
    let unit = normalize_unit(attrs.get("unit").map(String::as_str).unwrap_or(""));
    let kind = match hk_type {
        "HKQuantityTypeIdentifierBodyMass" => "weight",
        "HKQuantityTypeIdentifierHeight" => "height",
        "HKQuantityTypeIdentifierBodyMassIndex" => "bmi",
        "HKQuantityTypeIdentifierBodyFatPercentage" => "body_fat",
        "HKQuantityTypeIdentifierRestingHeartRate" => "resting_heart_rate",
        "HKQuantityTypeIdentifierBloodPressureSystolic" => "blood_pressure_systolic",
        "HKQuantityTypeIdentifierBloodPressureDiastolic" => "blood_pressure_diastolic",
        "HKQuantityTypeIdentifierBloodGlucose" => "blood_glucose",
        "HKQuantityTypeIdentifierBodyTemperature" => "body_temperature",
        "HKQuantityTypeIdentifierOxygenSaturation" => "oxygen_saturation",
        "HKQuantityTypeIdentifierWaistCircumference" => "waist_circumference",
        _ => return Ok(None),
    };
    Ok(Some((kind.into(), value, unit)))
}

fn sleep_from_attrs(
    hk_type: &str,
    attrs: &HashMap<String, String>,
) -> Option<(String, f64, String)> {
    if !matches!(
        hk_type,
        "HKQuantityTypeIdentifierTimeInBed" | "HKQuantityTypeIdentifierSleepDuration"
    ) {
        return None;
    }
    let unit = normalize_unit(attrs.get("unit").map(String::as_str).unwrap_or(""));
    if let Some(value) = attrs
        .get("value")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
    {
        let (value, unit) = crate::measurements::scale_sleep_import(value, &unit);
        if (0.0..=24.0).contains(&value) {
            return Some(("sleep".into(), value, unit));
        }
        return None;
    }
    let start = attrs
        .get("startDate")
        .and_then(|value| parse_apple_time(value).ok())?;
    let end = attrs
        .get("endDate")
        .and_then(|value| parse_apple_time(value).ok())?;
    let hours = (end - start).num_seconds() as f64 / 3600.0;
    if hours > 0.0 && hours <= 24.0 {
        Some(("sleep".into(), hours, "h".into()))
    } else {
        None
    }
}

fn workout_from_attrs(attrs: &HashMap<String, String>) -> Result<Option<UpsertWorkout>, AppError> {
    let Some(started_at) = attrs
        .get("startDate")
        .and_then(|value| parse_apple_time(value).ok())
    else {
        return Ok(None);
    };
    let activity = attrs
        .get("workoutActivityType")
        .map(String::as_str)
        .unwrap_or("");
    Ok(Some(UpsertWorkout {
        workout_type: map_workout(activity),
        started_at: Some(started_at),
        duration_seconds: duration_seconds(attrs),
        distance: None,
        distance_unit: None,
        calories: None,
        notes: attrs.get("sourceName").cloned(),
        exercises: Vec::new(),
    }))
}

fn map_workout(activity: &str) -> WorkoutType {
    let activity = activity.to_ascii_lowercase();
    if activity.contains("run") {
        WorkoutType::Running
    } else if activity.contains("cycl") || activity.contains("bike") {
        WorkoutType::Cycling
    } else if activity.contains("walk") || activity.contains("hike") {
        WorkoutType::Walking
    } else if activity.contains("swim") {
        WorkoutType::Swimming
    } else if activity.contains("strength") || activity.contains("functional") {
        WorkoutType::Strength
    } else if activity.contains("yoga")
        || activity.contains("flex")
        || activity.contains("cooldown")
    {
        WorkoutType::Mobility
    } else if activity.contains("soccer")
        || activity.contains("basket")
        || activity.contains("tennis")
        || activity.contains("sport")
    {
        WorkoutType::Sports
    } else {
        WorkoutType::Other
    }
}

fn duration_seconds(attrs: &HashMap<String, String>) -> Option<i32> {
    let duration = attrs.get("duration")?.parse::<f64>().ok()?;
    if !duration.is_finite() || duration < 0.0 {
        return None;
    }
    let unit = attrs
        .get("durationUnit")
        .map(String::as_str)
        .unwrap_or("min");
    let seconds = match unit {
        "s" | "sec" | "second" | "seconds" => duration,
        "hr" | "hour" | "hours" => duration * 3600.0,
        _ => duration * 60.0,
    };
    Some(seconds.round() as i32)
}

fn normalize_unit(unit: &str) -> String {
    match unit.trim() {
        "count/min" => "bpm".into(),
        "lb" | "lbs" => "lb".into(),
        "degC" | "°C" | "C" => "°C".into(),
        "degF" | "°F" | "F" => "°F".into(),
        "cm" | "m" | "kg" | "%" | "mmHg" | "mmol/L" | "mg/dL" => unit.trim().into(),
        "" => "unit".into(),
        other => other.chars().take(24).collect(),
    }
}

pub fn parse_apple_time(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_str(value.trim(), "%Y-%m-%d %H:%M:%S %z")
        .or_else(|_| DateTime::parse_from_rfc3339(value.trim()))
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|_| AppError::validation("Invalid Apple Health timestamp"))
}

fn tags(xml: &str, name: &str) -> Vec<HashMap<String, String>> {
    let open = format!("<{name} ");
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = xml[from..].find(&open) {
        let start = from + rel;
        let rest = &xml[start..];
        let close = rest
            .find("/>")
            .or_else(|| rest.find('>'))
            .unwrap_or(rest.len());
        out.push(parse_attrs(&rest[..close]));
        from = start + close + 1;
        if out.len() > 50_000 {
            break;
        }
    }
    out
}

fn parse_attrs(tag: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let bytes = tag.as_bytes();
    let mut i = 0usize;
    while i < tag.len() {
        while i < tag.len() && !bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        let key_start = i;
        while i < tag.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        if key_start == i {
            break;
        }
        let key = &tag[key_start..i];
        while i < tag.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= tag.len() || bytes[i] != b'=' {
            continue;
        }
        i += 1;
        while i < tag.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= tag.len() || bytes[i] != b'"' {
            continue;
        }
        i += 1;
        let value_start = i;
        while i < tag.len() && bytes[i] != b'"' {
            i += 1;
        }
        let value = tag[value_start..i].to_string();
        if i < tag.len() {
            i += 1;
        }
        attrs.insert(key.to_string(), value);
    }
    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_body_mass_and_skips_steps() {
        let xml = r#"<HealthData>
<Record type="HKQuantityTypeIdentifierBodyMass" unit="kg" value="81.4" startDate="2026-01-15 07:30:00 +0000" sourceName="Withings"/>
<Record type="HKQuantityTypeIdentifierStepCount" unit="count" value="3200" startDate="2026-01-15 07:30:00 +0000"/>
<Record type="HKQuantityTypeIdentifierRestingHeartRate" unit="count/min" value="58" startDate="2026-01-15 07:30:00 +0000"/>
<Workout workoutActivityType="HKWorkoutActivityTypeRunning" duration="32" durationUnit="min" startDate="2026-01-14 06:00:00 +0000"/>
</HealthData>"#;
        let parsed = parse(xml).unwrap();
        assert_eq!(parsed.measurements.len(), 2);
        assert_eq!(parsed.measurements[0].kind, "weight");
        assert_eq!(parsed.measurements[0].value, 81.4);
        assert_eq!(parsed.workouts.len(), 1);
        assert!(parsed.skipped >= 1);
    }

    #[test]
    fn parses_time_in_bed_hours() {
        let xml = r#"<HealthData>
<Record type="HKQuantityTypeIdentifierTimeInBed" unit="hr" value="7.5" startDate="2026-01-15 22:00:00 +0000" endDate="2026-01-16 05:30:00 +0000"/>
<Record type="HKCategoryTypeIdentifierSleepAnalysis" value="HKCategoryValueSleepAnalysisAsleepCore" startDate="2026-01-15 22:00:00 +0000" endDate="2026-01-16 05:30:00 +0000"/>
</HealthData>"#;
        let parsed = parse(xml).unwrap();
        assert_eq!(parsed.measurements.len(), 1);
        assert_eq!(parsed.measurements[0].kind, "sleep");
        assert!((parsed.measurements[0].value - 7.5).abs() < f64::EPSILON);
        assert_eq!(parsed.measurements[0].unit, "h");
    }

    #[test]
    fn rejects_unrelated_xml() {
        assert!(parse("<note>hello</note>").is_err());
    }
}
