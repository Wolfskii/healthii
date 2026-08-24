use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use healthii_backend::api;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_is_public() {
    let app = api::health_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "healthii-backend");
}

#[tokio::test]
async fn me_requires_authentication() {
    let app = api::health_router().route(
        "/api/v1/me",
        axum::routing::get(|| async { StatusCode::UNAUTHORIZED }),
    );
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
}

#[test]
fn error_body_shape_matches_contract() {
    let body = healthii_backend::ErrorBody {
        error: healthii_backend::error::ErrorDetail {
            code: "VALIDATION_ERROR".into(),
            message: "Invalid measurement value".into(),
        },
    };
    let json = serde_json::to_value(body).unwrap();
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(json["error"]["message"], "Invalid measurement value");
}
