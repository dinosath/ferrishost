use std::sync::Arc;
use tracing_subscriber;

use ferrishost_web::{build_router, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    tracing::info!("Starting ferrishost-web...");

    // Initialize app state
    let app_state = Arc::new(AppState::new().await?);

    // Build router
    let app = build_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
