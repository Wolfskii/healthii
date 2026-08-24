use std::sync::Arc;

use sqlx::PgPool;

use crate::{auth::rate_limit::LoginLimiter, config::Config, storage::Storage};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub storage: Storage,
    pub login_limiter: LoginLimiter,
}

impl AppState {
    pub fn new(config: Config, pool: PgPool, storage: Storage) -> Self {
        Self {
            config: Arc::new(config),
            pool,
            storage,
            login_limiter: LoginLimiter::default(),
        }
    }
}
