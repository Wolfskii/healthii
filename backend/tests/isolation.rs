use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use healthii_backend::{api, config::Config, db, state::AppState, storage};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())
}

async fn app() -> Option<axum::Router> {
    let database_url = database_url()?;
    let mut config = Config::for_tests();
    config.database_url = database_url;
    let pool = db::connect(&config.database_url).await.ok()?;
    db::migrate(&pool).await.ok()?;
    let storage = storage::build(&config.storage).ok()?;
    Some(api::router(AppState::new(config, pool, storage)))
}

async fn register(app: &axum::Router, email: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": "correct-horse-battery",
                        "display_name": "Tester"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    body["token"].as_str().unwrap().to_string()
}

async fn json(
    app: &axum::Router,
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"));
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let request = builder
        .body(
            body.map(|value| Body::from(value.to_string()))
                .unwrap_or_else(Body::empty),
        )
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

#[tokio::test]
async fn user_a_cannot_read_user_b_measurement() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("b-{suffix}@healthii.test")).await;

    let (status, created) = json(
        &app,
        "POST",
        "/api/v1/measurements",
        &token_a,
        Some(json!({
            "type": "weight",
            "value": 82.4,
            "unit": "kg"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();

    let (status, body) = json(
        &app,
        "GET",
        &format!("/api/v1/measurements/{id}"),
        &token_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");

    let (status, list) = json(&app, "GET", "/api/v1/measurements", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().map(|rows| rows.len()).unwrap_or(1), 0);
}

#[tokio::test]
async fn unauthenticated_users_cannot_create_measurements() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/measurements")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"type":"weight","value":80.0,"unit":"kg"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_a_cannot_read_user_b_lab_or_search_hit() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("lab-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("lab-b-{suffix}@healthii.test")).await;
    let marker = format!("marker-{suffix}");

    let (status, created) = json(
        &app,
        "POST",
        "/api/v1/labs",
        &token_a,
        Some(json!({
            "tested_on": "2026-01-15",
            "laboratory": marker,
            "results": [{
                "biomarker_code": "vitamin_d",
                "value": 70.0,
                "unit": "nmol/L"
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();

    let (status, body) = json(&app, "GET", &format!("/api/v1/labs/{id}"), &token_b, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");

    let (status, created_m) = json(
        &app,
        "POST",
        "/api/v1/measurements",
        &token_a,
        Some(json!({
            "type": "weight",
            "value": 80.0,
            "unit": "kg",
            "notes": marker
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(created_m["id"].as_str().is_some());

    let (status, search_b) = json(
        &app,
        "GET",
        &format!("/api/v1/search?q={marker}"),
        &token_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        search_b["items"]
            .as_array()
            .map(|rows| rows.len())
            .unwrap_or(1),
        0
    );

    let (status, search_a) = json(
        &app,
        "GET",
        &format!("/api/v1/search?q={marker}"),
        &token_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        search_a["items"]
            .as_array()
            .map(|rows| rows.len())
            .unwrap_or(0)
            >= 1
    );
}

#[tokio::test]
async fn document_file_requires_authentication() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/documents/{id}/file"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn import_assigns_rows_to_the_authenticated_user() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("imp-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("imp-b-{suffix}@healthii.test")).await;
    let foreign = uuid::Uuid::new_v4();

    let (status, result) = json(
        &app,
        "POST",
        "/api/v1/import",
        &token_a,
        Some(json!({
            "measurements": [{
                "id": foreign,
                "user_id": foreign,
                "type": "weight",
                "value": 71.2,
                "unit": "kg"
            }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(result["measurements"], 1);

    let (status, list_a) = json(&app, "GET", "/api/v1/measurements", &token_a, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_a.as_array().map(|rows| rows.len()).unwrap_or(0), 1);
    assert_ne!(list_a[0]["user_id"].as_str().unwrap(), foreign.to_string());

    let (status, list_b) = json(&app, "GET", "/api/v1/measurements", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_b.as_array().map(|rows| rows.len()).unwrap_or(1), 0);
}

#[tokio::test]
async fn import_csv_and_unauthenticated_import_are_scoped() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token = register(&app, &format!("csv-{suffix}@healthii.test")).await;
    let csv = "entity,id,occurred_at,type,value,unit,title\nmeasurement,,2026-08-01T00:00:00Z,weight,80.5,kg,\n";
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/csv")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "text/csv")
                .body(Body::from(csv))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let denied = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"measurements":[]}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn apple_health_import_is_scoped_to_the_caller() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("ah-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("ah-b-{suffix}@healthii.test")).await;
    let xml = r#"<HealthData><Record type="HKQuantityTypeIdentifierBodyMass" unit="kg" value="79.1" startDate="2026-01-15 07:30:00 +0000"/></HealthData>"#;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/apple-health")
                .header("authorization", format!("Bearer {token_a}"))
                .header("content-type", "application/xml")
                .body(Body::from(xml))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let (status, list_a) = json(&app, "GET", "/api/v1/measurements", &token_a, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_a.as_array().map(|rows| rows.len()).unwrap_or(0), 1);

    let (status, list_b) = json(&app, "GET", "/api/v1/measurements", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_b.as_array().map(|rows| rows.len()).unwrap_or(1), 0);
}

#[tokio::test]
async fn user_cannot_list_another_users_sessions() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("sess-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("sess-b-{suffix}@healthii.test")).await;
    let (status, sessions) = json(&app, "GET", "/api/v1/sessions", &token_a, None).await;
    assert_eq!(status, StatusCode::OK);
    let id = sessions[0]["id"].as_str().unwrap();
    let (status, _) = json(
        &app,
        "DELETE",
        &format!("/api/v1/sessions/{id}"),
        &token_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn apple_health_import_requires_auth() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let denied = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/apple-health")
                .header("content-type", "application/xml")
                .body(Body::from("<HealthData></HealthData>"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_connect_csv_import_is_scoped_to_the_caller() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("hc-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("hc-b-{suffix}@healthii.test")).await;
    let csv = "Start time,Weight (kg)\n2026-01-15 07:30:00,77.4\n";
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/csv")
                .header("authorization", format!("Bearer {token_a}"))
                .header("content-type", "text/csv")
                .body(Body::from(csv))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let (status, list_a) = json(&app, "GET", "/api/v1/measurements", &token_a, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_a.as_array().map(|rows| rows.len()).unwrap_or(0), 1);

    let (status, list_b) = json(&app, "GET", "/api/v1/measurements", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_b.as_array().map(|rows| rows.len()).unwrap_or(0), 0);
}

#[tokio::test]
async fn delete_account_requires_password() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token = register(&app, &format!("del-{suffix}@healthii.test")).await;
    let (status, _) = json(
        &app,
        "POST",
        "/api/v1/measurements",
        &token,
        Some(json!({
            "type": "weight",
            "value": 80.0,
            "unit": "kg"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = json(
        &app,
        "DELETE",
        "/api/v1/me",
        &token,
        Some(json!({ "password": "not-the-password-xx" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = json(
        &app,
        "DELETE",
        "/api/v1/me",
        &token,
        Some(json!({ "password": "correct-horse-battery" })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = json(&app, "GET", "/api/v1/me", &token, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn notes_are_scoped_to_the_caller() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("note-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("note-b-{suffix}@healthii.test")).await;
    let (status, created) = json(
        &app,
        "POST",
        "/api/v1/notes",
        &token_a,
        Some(json!({
            "title": "Clinic question",
            "body": "Ask about vitamin D next visit"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();

    let (status, list_b) = json(&app, "GET", "/api/v1/notes", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_b.as_array().map(|rows| rows.len()).unwrap_or(0), 0);

    let (status, _) = json(
        &app,
        "DELETE",
        &format!("/api/v1/notes/{id}"),
        &token_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, search) = json(&app, "GET", "/api/v1/search?q=vitamin", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(search["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn activity_is_scoped_and_omits_health_values() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("act-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("act-b-{suffix}@healthii.test")).await;
    let (status, events) = json(&app, "GET", "/api/v1/activity", &token_a, None).await;
    assert_eq!(status, StatusCode::OK);
    let rows = events.as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row.get("action").is_some());
        assert!(row.get("ip").is_none());
        let dumped = row.to_string();
        assert!(!dumped.contains("82.4"));
    }
    let (status, other) = json(&app, "GET", "/api/v1/activity", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    let other_ids: Vec<&str> = other
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect();
    for row in rows {
        let id = row["id"].as_str().unwrap();
        assert!(!other_ids.contains(&id));
    }
}

#[tokio::test]
async fn update_me_is_scoped_to_the_caller() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let suffix = uuid::Uuid::new_v4();
    let token_a = register(&app, &format!("me-a-{suffix}@healthii.test")).await;
    let token_b = register(&app, &format!("me-b-{suffix}@healthii.test")).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/me")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "display_name": "Ada" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let (status, _) = json(
        &app,
        "PUT",
        "/api/v1/me",
        &token_a,
        Some(json!({ "timezone": "nope" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, me_a) = json(
        &app,
        "PUT",
        "/api/v1/me",
        &token_a,
        Some(json!({
            "display_name": "Ada",
            "timezone": "Europe/Warsaw"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me_a["display_name"], "Ada");
    assert_eq!(me_a["timezone"], "Europe/Warsaw");

    let (status, me_b) = json(&app, "GET", "/api/v1/me", &token_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me_b["display_name"], "Tester");
    assert_eq!(me_b["timezone"], "UTC");
}
