use healthii_backend::{api, config::Config, db, state::AppState, storage, telemetry};

#[tokio::main]
async fn main() {
    telemetry::init();

    if let Err(error) = run().await {
        tracing::error!(error = %error, "backend failed to start");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;

    let storage = storage::build(&config.storage)?;
    let state = AppState::new(config.clone(), pool, storage);
    let app = api::router(state.clone());
    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port)).await?;

    tracing::info!(
        host = %config.host,
        port = config.port,
        "healthii backend listening"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutdown signal received");
}
