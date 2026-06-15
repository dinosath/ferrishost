use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use ferrishost_core::ModuleDescriptor;
use k8s_openapi::api::batch::v1::Job;

use crate::k8s::JobStatusResponse;
use crate::state::AppState;

/// GET /api/modules — list all available modules.
///
/// If Artifactory is configured, the catalog is fetched from there.
/// Otherwise falls back to a built-in static list.
pub async fn list_modules(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ModuleDescriptor>> {
    // Try Artifactory first
    if let Ok(Some(catalog)) = state.artifactory.fetch_catalog().await {
        let modules: Vec<ModuleDescriptor> = catalog
            .modules
            .into_iter()
            .map(|m| ModuleDescriptor {
                id: m.id,
                name: m.name,
                description: m.description,
                category: m.category,
                version: m.version,
                installed: false, // TODO: query installed state from cluster
            })
            .collect();
        return Json(modules);
    }

    // Fallback: static list
    Json(vec![
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
    ])
}

/// GET /api/modules/:id — fetch metadata for a single module.
pub async fn get_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ModuleDescriptor>, StatusCode> {
    if let Ok(Some(catalog)) = state.artifactory.fetch_catalog().await {
        if let Some(cm) = catalog.modules.into_iter().find(|m| m.id == id) {
            return Ok(Json(ModuleDescriptor {
                id: cm.id,
                name: cm.name,
                description: cm.description,
                category: cm.category,
                version: cm.version,
                installed: false,
            }));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

/// POST /api/modules/:id/install — install a module.
///
/// If a Kubernetes client is available, the install Job manifest is fetched
/// from Artifactory (if configured) and a Job is created. Otherwise a
/// placeholder response is returned (useful during development / testing).
pub async fn install_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    // If no K8s client is available, return a placeholder (e.g. during tests)
    let Some(k8s) = state.k8s.as_ref() else {
        tracing::info!("Install requested for {id} (no K8s client — returning placeholder)");
        return Json(serde_json::json!({
            "status": "installing",
            "module": id,
        }));
    };

    // Try to fetch the job manifest from Artifactory
    let raw_yaml = state
        .artifactory
        .fetch_install_job(&id)
        .await
        .unwrap_or(None);

    match raw_yaml {
        Some(yaml) => {
            // Template and create the Job
            let templated = template_manifest(&yaml, &id);
            if let Ok(job) = serde_yaml::from_str::<Job>(&templated) {
                match k8s.create_job("ferrishost", &job).await {
                    Ok(job_name) => {
                        return Json(serde_json::json!({
                            "status": "installing",
                            "module": id,
                            "jobName": job_name,
                        }));
                    }
                    Err(e) => {
                        tracing::error!("Failed to create Job for {id}: {e}");
                        return Json(serde_json::json!({
                            "status": "error",
                            "module": id,
                            "error": format!("{e}"),
                        }));
                    }
                }
            }
        }
        None => {
            tracing::info!("No install manifest found for {id}; returning placeholder");
        }
    }

    Json(serde_json::json!({
        "status": "installing",
        "module": id,
        "note": "Artifactory not configured or manifest not found"
    }))
}

/// POST /api/modules/:id/uninstall — uninstall a module.
pub async fn uninstall_module(
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    tracing::info!("Uninstalling module: {id}");
    Json(serde_json::json!({ "status": "uninstalling", "module": id }))
}

/// GET /api/jobs/:name — poll the status of an install Job.
pub async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    let k8s = state.k8s.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let status = k8s
        .get_job_status("ferrishost", &name)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(status))
}

// ---------------------------------------------------------------------------
// Manifest templating
// ---------------------------------------------------------------------------

/// Very simple YAML template engine that replaces `{{ key }}` placeholders.
fn template_manifest(yaml: &str, module_id: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    yaml.replace("{{ module_id }}", module_id)
        .replace("{{ timestamp }}", &timestamp.to_string())
        .replace("{{ namespace }}", "ferrishost")
        .replace("{{ domain }}", "ferrishost.homelab.local")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manifest_replaces_placeholders() {
        let yaml = r#"
metadata:
  name: install-{{ module_id }}-{{ timestamp }}
  namespace: {{ namespace }}
"#;
        let result = template_manifest(yaml, "headscale");
        assert!(result.contains("install-headscale-"));
        assert!(result.contains("ferrishost"));
        assert!(!result.contains("{{ module_id }}"));
        assert!(!result.contains("{{ timestamp }}"));
    }

    #[test]
    fn test_template_manifest_preserves_normal_content() {
        let yaml = r#"
apiVersion: batch/v1
kind: Job
metadata:
  name: test-job
"#;
        let result = template_manifest(yaml, "test");
        assert!(result.contains("apiVersion: batch/v1"));
        assert!(result.contains("kind: Job"));
        assert!(result.contains("test-job"));
    }
}
