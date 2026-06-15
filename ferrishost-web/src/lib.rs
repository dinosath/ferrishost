use axum::{
    Router,
    body::Body,
    extract::OriginalUri,
    http::StatusCode,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;
use std::{env, path::PathBuf};
use tower_http::cors::CorsLayer;

pub mod k8s;
pub mod routes;
pub mod state;

use state::AppState;

/// Default directory for compiled frontend assets.
const DEFAULT_STATIC_DIR: &str = "/srv/static";

/// Build the application router with all API routes and static file serving.
pub fn build_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // API routes
        .route("/api/status", get(routes::status::get_status))
        .route("/api/gpu", get(routes::status::get_gpu_status))
        .route("/api/setup", get(routes::setup::get_setup_state))
        .route("/api/setup", post(routes::setup::update_setup_state))
        .route("/api/modules", get(routes::modules::list_modules))
        .route("/api/modules/:id", get(routes::modules::get_module))
        .route(
            "/api/modules/:id/install",
            post(routes::modules::install_module),
        )
        .route(
            "/api/modules/:id/uninstall",
            post(routes::modules::uninstall_module),
        )
        .route("/api/jobs/:name", get(routes::modules::get_job_status))
        // Health checks
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        // Static file serving + SPA fallback (catch-all for GET)
        .fallback(get(spa_or_static_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

/// Health check — always responds 200.
async fn healthz() -> StatusCode {
    StatusCode::OK
}

/// Readiness check — always responds 200 for now.
async fn readyz() -> StatusCode {
    StatusCode::OK
}

/// Serve a static file, or fall back to index.html for client-side routing.
async fn spa_or_static_handler(
    OriginalUri(uri): OriginalUri,
) -> Response<Body> {
    let static_dir = env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_STATIC_DIR));

    let path = uri.path().trim_start_matches('/');
    let file_path = static_dir.join(path);

    // If the file exists, serve it
    if file_path.exists() && file_path.is_file() {
        match tokio::fs::read(&file_path).await {
            Ok(bytes) => {
                let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", mime.to_string())
                    .body(Body::from(bytes))
                    .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR).into_response());
            }
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        }
    }

    // SPA fallback: serve index.html
    match tokio::fs::read_to_string(static_dir.join("index.html")).await {
        Ok(html) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .body(Body::from(html))
            .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR).into_response()),
        Err(_) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .body(Body::from(
                r#"<!DOCTYPE html><html lang="en"><body>
                <h1>FerrisHost</h1>
                <p>Frontend not built yet. Run <code>npm run build</code> in ferrishost-web/frontend/.</p>
                </body></html>"#,
            ))
            .unwrap(),
    }
}
