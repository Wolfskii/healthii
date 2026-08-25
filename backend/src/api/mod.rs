mod openapi;

use std::time::Duration;

use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{header, HeaderName, HeaderValue, Method, Request},
    routing::{delete, get, post, put},
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

use crate::{
    appointments, audit, auth, charts, dashboard, documents, export, health, import, laboratory,
    measurements, medications, notes, profile, search, state::AppState, symptoms, timeline, users,
    workouts,
};

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
            .layer(DefaultBodyLimit::max(32 * 1024 * 1024)),
    )
}

fn v1_router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/password", put(users::change_password))
        .route(
            "/me",
            get(users::me)
                .put(users::update_me)
                .delete(users::delete_account),
        )
        .route("/activity", get(audit::list))
        .route("/sessions", get(users::list_sessions))
        .route("/sessions/{id}", delete(users::revoke_session))
        .route("/profile", get(profile::get).put(profile::update))
        .route("/dashboard", get(dashboard::get_dashboard))
        .route("/timeline", get(timeline::get))
        .route("/search", get(search::get))
        .route("/charts", get(charts::get))
        .route("/export", get(export::get))
        .route("/import", post(import::json))
        .route("/import/csv", post(import::csv))
        .route("/import/apple-health", post(import::apple_health))
        .route(
            "/measurements",
            get(measurements::list).post(measurements::create),
        )
        .route(
            "/measurements/blood-pressure",
            post(measurements::create_blood_pressure),
        )
        .route(
            "/measurements/{id}",
            get(measurements::get)
                .put(measurements::update)
                .delete(measurements::delete),
        )
        .route("/labs/biomarkers", get(laboratory::biomarkers))
        .route("/labs", get(laboratory::list).post(laboratory::create))
        .route(
            "/labs/{id}",
            get(laboratory::get)
                .put(laboratory::update)
                .delete(laboratory::delete),
        )
        .route("/workouts", get(workouts::list).post(workouts::create))
        .route(
            "/workouts/{id}",
            get(workouts::get)
                .put(workouts::update)
                .delete(workouts::delete),
        )
        .route(
            "/medications",
            get(medications::list).post(medications::create),
        )
        .route("/medications/{id}", delete(medications::delete))
        .route("/symptoms", get(symptoms::list).post(symptoms::create))
        .route("/symptoms/{id}", delete(symptoms::delete))
        .route("/notes", get(notes::list).post(notes::create))
        .route("/notes/{id}", delete(notes::delete))
        .route(
            "/appointments",
            get(appointments::list).post(appointments::create),
        )
        .route("/appointments/{id}", delete(appointments::delete))
        .merge(documents::router())
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
