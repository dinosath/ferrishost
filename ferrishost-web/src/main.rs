use axum::{
    routing::{get, post},
    Router, Json, State,
};
use ferrishost_core::{GpuStatus, HostInfo, ClusterStatus, SetupState, ModuleDescriptor};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod routes;
mod k8s;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("Starting ferrishost-web...");

    // Initialize app state
    let app_state = Arc::new(AppState::new().await?);

    // Build router
    let app = Router::new()
        .route("/api/status", get(routes::status::get_status))
        .route("/api/gpu", get(routes::status::get_gpu_status))
        .route("/api/setup", get(routes::setup::get_setup_state))
        .route("/api/setup", post(routes::setup::update_setup_state))
        .route("/api/modules", get(routes::modules::list_modules))
        .route("/api/modules/:id/install", post(routes::modules::install_module))
        .route("/api/modules/:id/uninstall", post(routes::modules::uninstall_module))
        .fallback(static_files_handler)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn static_files_handler() -> &'static str {
    "ferrishost-web static files would be served here"
}
