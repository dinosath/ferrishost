use anyhow::{Context, Result, anyhow};

/// The version of ferrishost-web image to deploy.
///
/// This is the current ferrishost-cli version; the web image is tagged the same
/// so that the CLI always deploys a matching web image.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default Helm chart reference (OCI).
const CHART_REF: &str = "oci://ghcr.io/dinosath/charts/ferrishost-web";

/// Deploy ferrishost-web into a k3s cluster via Helm.
pub struct WebDeployer {
    image_tag: String,
    namespace: String,
    ingress_host: String,
    chart_ref: String,
}

impl WebDeployer {
    /// Create a new deployer that will install/upgrade the web service.
    ///
    /// `ingress_host` is the DNS name the service will be reachable at
    /// (e.g. "ferrishost.homelab.local").
    pub fn new(ingress_host: &str) -> Self {
        Self {
            image_tag: VERSION.to_string(),
            namespace: "ferrishost".to_string(),
            ingress_host: ingress_host.to_string(),
            chart_ref: CHART_REF.to_string(),
        }
    }

    /// Override the image tag (useful for development / testing).
    #[allow(dead_code)]
    pub fn with_image_tag(mut self, tag: &str) -> Self {
        self.image_tag = tag.to_string();
        self
    }

    /// Override the Helm chart reference (useful for local testing).
    #[allow(dead_code)]
    pub fn with_chart_ref(mut self, chart: &str) -> Self {
        self.chart_ref = chart.to_string();
        self
    }

    /// Deploy (or upgrade) ferrishost-web and wait for it to become ready.
    pub async fn deploy(&self) -> Result<()> {
        tracing::info!(
            "Deploying ferrishost-web (tag={}, namespace={})",
            self.image_tag,
            self.namespace
        );

        self.helm_install().await?;
        self.wait_for_ready().await?;

        tracing::info!("ferrishost-web is ready");
        Ok(())
    }

    // ------------------------------------------------------------------
    // Helm helpers
    // ------------------------------------------------------------------

    async fn helm_install(&self) -> Result<()> {
        let output = tokio::process::Command::new("helm")
            .args([
                "upgrade",
                "--install",
                "ferrishost-web",
                &self.chart_ref,
                "--namespace",
                &self.namespace,
                "--create-namespace",
                "--set",
                &format!("image.tag={}", self.image_tag),
                "--set",
                &format!("ingress.host={}", self.ingress_host),
                "--wait",
                "--timeout",
                "5m",
            ])
            .output()
            .await
            .context("failed to spawn helm")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::error!("Helm install failed:\nstdout: {stdout}\nstderr: {stderr}");
            return Err(anyhow!("helm upgrade --install failed: {stderr}"));
        }

        tracing::info!("Helm release ferrishost-web installed/upgraded");
        Ok(())
    }

    async fn wait_for_ready(&self) -> Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args([
                "rollout",
                "status",
                "deployment/ferrishost-web",
                "-n",
                &self.namespace,
                "--timeout=300s",
            ])
            .output()
            .await
            .context("failed to spawn kubectl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("kubectl rollout status failed: {stderr}"));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        // The version should be a non-empty semver string
        assert!(!VERSION.is_empty(), "CARGO_PKG_VERSION should not be empty");
        assert!(
            VERSION.contains('.'),
            "version should be semver (contain dots): {VERSION}"
        );
    }

    #[test]
    fn test_deployer_defaults() {
        let d = WebDeployer::new("test.example.com");
        assert_eq!(d.image_tag, VERSION);
        assert_eq!(d.namespace, "ferrishost");
        assert_eq!(d.ingress_host, "test.example.com");
        assert_eq!(d.chart_ref, CHART_REF);
    }

    #[test]
    fn test_deployer_overrides() {
        let d = WebDeployer::new("test.example.com")
            .with_image_tag("1.2.3")
            .with_chart_ref("/tmp/test-chart");
        assert_eq!(d.image_tag, "1.2.3");
        assert_eq!(d.chart_ref, "/tmp/test-chart");
    }
}
