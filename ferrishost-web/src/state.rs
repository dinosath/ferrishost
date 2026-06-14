use anyhow::Result;
use ferrishost_core::SetupState;

pub struct AppState {
    pub setup_state: SetupState,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        Ok(AppState {
            setup_state: SetupState {
                domain: None,
                tls_mode: ferrishost_core::TlsMode::SelfSigned,
                admin_username: None,
                admin_password_hash: None,
                timezone: None,
                completed: false,
            },
        })
    }
}
