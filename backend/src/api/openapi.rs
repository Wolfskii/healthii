use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    auth::{AuthResponse, LoginRequest, RegisterRequest},
    dashboard::DashboardResponse,
    error::ErrorBody,
    health::HealthResponse,
    users::User,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Healthii API",
        version = "1.0.0",
        description = "Privacy-first personal health API. All user-owned resources are scoped to the authenticated user. Healthii does not provide medical diagnosis."
    ),
    paths(
        crate::health::live,
        crate::health::ready,
        crate::auth::register,
        crate::auth::login,
        crate::auth::logout,
        crate::users::me,
        crate::dashboard::get_dashboard,
    ),
    components(schemas(
        HealthResponse,
        RegisterRequest,
        LoginRequest,
        AuthResponse,
        User,
        DashboardResponse,
        ErrorBody,
        crate::error::ErrorDetail,
        crate::dashboard::DashboardWidget,
    )),
    tags(
        (name = "ops", description = "Process health and readiness"),
        (name = "auth", description = "Registration and sessions"),
        (name = "users", description = "Current user profile"),
        (name = "dashboard", description = "Health overview widgets"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}
