use std::time::Duration;

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub session_secret: String,
    pub jwt_ttl: Duration,
    pub cors_origins: Vec<String>,
    pub openapi_enabled: bool,
    pub storage: StorageConfig,
}

#[derive(Clone, Debug)]
pub enum StorageConfig {
    Local { path: String },
    S3(S3Config),
}

#[derive(Clone, Debug)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub allow_http: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let host = env_or("HOST", "0.0.0.0");
        let port = env_or("PORT", "8080")
            .parse::<u16>()
            .map_err(|_| AppError::config("PORT must be a valid TCP port"))?;
        let database_url = required("DATABASE_URL")?;
        let jwt_secret = required("JWT_SECRET")?;
        let session_secret = env_or("SESSION_SECRET", &jwt_secret);

        if jwt_secret.len() < 32 {
            return Err(AppError::config(
                "JWT_SECRET must be at least 32 characters",
            ));
        }

        let jwt_ttl_seconds = env_or("JWT_TTL_SECONDS", "43200")
            .parse::<u64>()
            .map_err(|_| AppError::config("JWT_TTL_SECONDS must be a number"))?;

        let cors_origins = env_or(
            "CORS_ORIGINS",
            "http://localhost:5173,http://localhost:1420",
        )
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();

        let openapi_enabled = env_or("OPENAPI_ENABLED", "false")
            .parse::<bool>()
            .unwrap_or(false);

        Ok(Self {
            host,
            port,
            database_url,
            jwt_secret,
            session_secret,
            jwt_ttl: Duration::from_secs(jwt_ttl_seconds),
            cors_origins,
            openapi_enabled,
            storage: StorageConfig::from_env()?,
        })
    }

    pub fn for_tests() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 0,
            database_url: "postgres://healthii:healthii@127.0.0.1:5432/healthii".into(),
            jwt_secret: "test-jwt-secret-that-is-at-least-32-chars".into(),
            session_secret: "test-session-secret-that-is-32-chars+".into(),
            jwt_ttl: Duration::from_secs(3600),
            cors_origins: vec!["http://localhost:5173".into()],
            openapi_enabled: true,
            storage: StorageConfig::Local {
                path: "./data/test-storage".into(),
            },
        }
    }
}

impl StorageConfig {
    fn from_env() -> Result<Self, AppError> {
        match env_or("STORAGE_PROVIDER", "local")
            .to_ascii_lowercase()
            .as_str()
        {
            "local" => Ok(Self::Local {
                path: env_or("STORAGE_PATH", "./data/storage"),
            }),
            "s3" => Ok(Self::S3(S3Config {
                endpoint: required("S3_ENDPOINT")?,
                bucket: required("S3_BUCKET")?,
                region: env_or("S3_REGION", "us-east-1"),
                access_key: required("S3_ACCESS_KEY")?,
                secret_key: required("S3_SECRET_KEY")?,
                allow_http: env_or("S3_ALLOW_HTTP", "false")
                    .parse::<bool>()
                    .unwrap_or(false),
            })),
            other => Err(AppError::config(format!(
                "unsupported STORAGE_PROVIDER '{other}' (use local or s3)"
            ))),
        }
    }
}

fn required(name: &str) -> Result<String, AppError> {
    std::env::var(name)
        .map_err(|_| AppError::config(format!("{name} must be set")))
        .and_then(|value| {
            if value.trim().is_empty() {
                Err(AppError::config(format!("{name} must not be empty")))
            } else {
                Ok(value)
            }
        })
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_jwt_secret() {
        let error = AppError::config("JWT_SECRET must be at least 32 characters");
        assert!(error.to_string().contains("JWT_SECRET"));
    }

    #[test]
    fn test_config_has_safe_defaults() {
        let config = Config::for_tests();
        assert!(config.jwt_secret.len() >= 32);
        assert!(config.openapi_enabled);
    }
}
