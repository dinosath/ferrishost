use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use ferrishost_web::{build_router, state::AppState};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

async fn make_app() -> axum::Router {
    let state = Arc::new(AppState::new().await.expect("AppState::new"));
    build_router(state)
}

// ---------------------------------------------------------------------------
// GET /api/status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_status_returns_200() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_status_body_is_valid_json() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert!(json.get("k3s_installed").is_some());
    assert!(json.get("nodes").is_some());
}

// ---------------------------------------------------------------------------
// GET /api/gpu
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_gpu_returns_200() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/gpu")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_gpu_body_has_expected_fields() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/gpu")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert!(json.get("detected").is_some());
    assert!(json.get("nvidia_available").is_some());
    assert!(json.get("amd_available").is_some());
}

// ---------------------------------------------------------------------------
// GET /api/setup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_setup_returns_200() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_setup_body_has_completed_false() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert_eq!(json["completed"], false);
}

// ---------------------------------------------------------------------------
// POST /api/setup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_setup_echoes_body() {
    let app = make_app().await;
    let payload = serde_json::json!({
        "domain": "home.example.com",
        "tls_mode": "letsencrypt",
        "admin_username": "admin",
        "admin_password_hash": null,
        "timezone": "UTC",
        "completed": true
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert_eq!(json["domain"], "home.example.com");
    assert_eq!(json["completed"], true);
}

// ---------------------------------------------------------------------------
// GET /api/modules
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_modules_returns_200() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/modules")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_modules_returns_array() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/modules")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert!(json.is_array());
    let arr = json.as_array().unwrap();
    assert!(!arr.is_empty(), "modules list should not be empty");
    // Each module should have id, name, category
    for module in arr {
        assert!(module.get("id").is_some());
        assert!(module.get("name").is_some());
        assert!(module.get("category").is_some());
    }
}

// ---------------------------------------------------------------------------
// POST /api/modules/:id/install
// ---------------------------------------------------------------------------

#[tokio::test]
async fn install_module_returns_200() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/modules/headscale/install")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn install_module_response_contains_module_id() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/modules/seafile/install")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert_eq!(json["module"], "seafile");
    assert_eq!(json["status"], "installing");
}

// ---------------------------------------------------------------------------
// POST /api/modules/:id/uninstall
// ---------------------------------------------------------------------------

#[tokio::test]
async fn uninstall_module_returns_200() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/modules/headscale/uninstall")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn uninstall_module_response_contains_module_id() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/modules/headscale/uninstall")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    assert_eq!(json["module"], "headscale");
    assert_eq!(json["status"], "uninstalling");
}

// ---------------------------------------------------------------------------
// Fallback / 404
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unknown_route_returns_fallback() {
    let app = make_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/some/unknown/path")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // The fallback handler returns 200 with a static string (serves SPA)
    assert_eq!(response.status(), StatusCode::OK);
}
