use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub mod k8s;
pub mod routes;
pub mod state;

use state::AppState;

pub fn build_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/status", get(routes::status::get_status))
        .route("/api/gpu", get(routes::status::get_gpu_status))
        .route("/api/setup", get(routes::setup::get_setup_state))
        .route("/api/setup", post(routes::setup::update_setup_state))
        .route("/api/modules", get(routes::modules::list_modules))
        .route(
            "/api/modules/:id/install",
            post(routes::modules::install_module),
        )
        .route(
            "/api/modules/:id/uninstall",
            post(routes::modules::uninstall_module),
        )
        .fallback(static_files_handler)
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

async fn static_files_handler() -> &'static str {
    "ferrishost-web static files would be served here"
}
