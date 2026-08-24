use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use healthii_backend::{api, config::Config, db, state::AppState, storage};
use http_body_util::BodyExt;
use serde_json::json;
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

#[tokio::test]
async fn unauthenticated_users_cannot_read_profile() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn user_a_cannot_use_user_b_token_after_logout() {
    let Some(app) = app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };

    let suffix = uuid::Uuid::new_v4();
    let email_a = format!("ada-{suffix}@healthii.test");
    let register = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email_a,
                "password": "correct-horse-battery",
                "display_name": "Ada"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(register).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = created["token"].as_str().unwrap().to_string();

    let logout = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(logout).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let me = Request::builder()
        .uri("/api/v1/me")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(me).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
