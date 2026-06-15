use anyhow::Result;
use ferrishost_core::SetupState;
use std::env;

use crate::k8s::K8sClient;

/// Shared application state available to all request handlers.
pub struct AppState {
    pub setup_state: SetupState,
    pub k8s: Option<K8sClient>,
    pub artifactory: ArtifactoryClient,
}

/// Client for fetching the module catalog and install manifests from JFrog
/// Artifactory.
#[derive(Debug, Clone)]
pub struct ArtifactoryClient {
    pub base_url: String,
    pub repository: String,
    pub anonymous: bool,
}

impl ArtifactoryClient {
    pub fn from_env() -> Self {
        Self {
            base_url: env::var("ARTIFACTORY_URL").unwrap_or_default(),
            repository: env::var("ARTIFACTORY_REPO").unwrap_or_else(|_| "ferrishost-modules".into()),
            anonymous: env::var("ARTIFACTORY_ANONYMOUS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
        }
    }

    /// Fetch the module catalog index from Artifactory.
    pub async fn fetch_catalog(&self) -> Result<Option<Catalog>> {
        if self.base_url.is_empty() {
            return Ok(None);
        }
        let url = format!("{}/{}/catalog.json", self.base_url, self.repository);
        let resp = reqwest::get(&url).await?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let catalog: Catalog = resp.json().await?;
        Ok(Some(catalog))
    }

    /// Fetch the install Job manifest YAML for a given module.
    pub async fn fetch_install_job(&self, module_id: &str) -> Result<Option<String>> {
        if self.base_url.is_empty() {
            return Ok(None);
        }
        let url = format!("{}/{}/{}/install-job.yaml", self.base_url, self.repository, module_id);
        let resp = reqwest::get(&url).await?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let text = resp.text().await?;
        Ok(Some(text))
    }
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let k8s = K8sClient::new().await.ok();

        Ok(AppState {
            setup_state: SetupState {
                domain: None,
                tls_mode: ferrishost_core::TlsMode::SelfSigned,
                admin_username: None,
                admin_password_hash: None,
                timezone: None,
                completed: false,
            },
            k8s,
            artifactory: ArtifactoryClient::from_env(),
        })
    }
}

// ---------------------------------------------------------------------------
// Artifactory API types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Catalog {
    pub version: String,
    pub modules: Vec<CatalogModule>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatalogModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub icon: Option<String>,
    pub install_job: Option<String>,
}
