mod openapi;

use std::time::Duration;

use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{header, HeaderName, HeaderValue, Method, Request},
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{auth, dashboard, health, state::AppState, users};

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state.config.cors_origins);
    let openapi_enabled = state.config.openapi_enabled;

    let mut app = Router::new()
        .route("/health", get(health::live))
        .route("/ready", get(health::ready))
        .route("/metrics", get(metrics))
        .nest("/api/v1", v1_router())
        .with_state(state);

    if openapi_enabled {
        app =
            app.merge(SwaggerUi::new("/docs").url("/api/openapi.json", openapi::ApiDoc::openapi()));
    }

    app.layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(
                REQUEST_ID_HEADER.clone(),
                MakeRequestUuid,
            ))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<Body>| {
                        let request_id = request
                            .headers()
                            .get(&REQUEST_ID_HEADER)
                            .and_then(|value| value.to_str().ok())
                            .unwrap_or("-");
                        tracing::info_span!(
                            "http",
                            service = "healthii-backend",
                            request_id,
                            method = %request.method(),
                            route = %request.uri().path(),
                        )
                    })
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Millis)
                            .include_headers(false),
                    ),
            )
            .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
            .layer(cors)
            .layer(DefaultBodyLimit::max(2 * 1024 * 1024)),
    )
}

fn v1_router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/me", get(users::me))
        .route("/dashboard", get(dashboard::get_dashboard))
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(parsed))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(60 * 60))
}

async fn metrics() -> String {
    [
        "# HELP healthii_up 1 if the process is running",
        "# TYPE healthii_up gauge",
        "healthii_up 1",
        "",
    ]
    .join("\n")
}

pub fn health_router() -> Router {
    Router::new().route("/health", get(health::live))
}
