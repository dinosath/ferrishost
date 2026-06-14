use axum::Json;
use ferrishost_core::{SetupState, TlsMode};

pub async fn get_setup_state() -> Json<SetupState> {
    Json(SetupState {
        domain: None,
        tls_mode: TlsMode::SelfSigned,
        admin_username: None,
        admin_password_hash: None,
        timezone: None,
        completed: false,
    })
}

pub async fn update_setup_state(Json(state): Json<SetupState>) -> Json<SetupState> {
    tracing::info!("Updating setup state: {:?}", state);
    Json(state)
}
