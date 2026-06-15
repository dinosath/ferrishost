use std::env;
use std::sync::Arc;
use tracing_subscriber;

use ferrishost_web::{build_router, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (respect RUST_LOG env var if set)
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    tracing::info!("Starting ferrishost-web...");

    // Log static file directory
    let static_dir =
        env::var("STATIC_DIR").unwrap_or_else(|_| "/srv/static".to_string());
    tracing::info!("Serving static files from: {static_dir}");

    // Initialize app state (K8s client, Artifactory client, etc.)
    let app_state = Arc::new(AppState::new().await?);

    // Build router
    let app = build_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
