use axum::Json;
use axum::extract::Path;
use ferrishost_core::ModuleDescriptor;

pub async fn list_modules() -> Json<Vec<ModuleDescriptor>> {
    let modules = vec![
        ModuleDescriptor {
            id: "headscale".to_string(),
            name: "Headscale".to_string(),
            description: "Private mesh VPN and remote access".to_string(),
            category: "vpn".to_string(),
            version: "0.22.0".to_string(),
            installed: false,
        },
        ModuleDescriptor {
            id: "seafile".to_string(),
            name: "Seafile".to_string(),
            description: "File sync and sharing".to_string(),
            category: "app".to_string(),
            version: "9.0.0".to_string(),
            installed: false,
        },
    ];
    Json(modules)
}

pub async fn install_module(
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    tracing::info!("Installing module: {}", id);
    Json(serde_json::json!({ "status": "installing", "module": id }))
}

pub async fn uninstall_module(
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    tracing::info!("Uninstalling module: {}", id);
    Json(serde_json::json!({ "status": "uninstalling", "module": id }))
}
