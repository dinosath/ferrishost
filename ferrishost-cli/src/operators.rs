use anyhow::{Context, Result};
use ferrishost_core::GpuInfo;
use std::collections::HashSet;
use tokio::io::AsyncWriteExt;

// ---------------------------------------------------------------------------
// Embedded Kubernetes manifests
// ---------------------------------------------------------------------------

const CERT_MANAGER_MANIFEST: &str =
    include_str!("../../manifests/cert-manager.yaml");

const RBAC_MANIFEST: &str =
    include_str!("../../manifests/ferrishost-core-rbac.yaml");

const NVIDIA_MANIFEST: &str =
    include_str!("../../manifests/nvidia-device-plugin.yaml");

const AMD_MANIFEST: &str =
    include_str!("../../manifests/amd-device-plugin.yaml");

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

pub struct Operators;

impl Operators {
    /// Install cert-manager CRDs, namespace, and a self-signed ClusterIssuer.
    pub async fn install_cert_manager() -> Result<()> {
        tracing::info!("Installing cert-manager...");
        Self::kubectl_apply(CERT_MANAGER_MANIFEST)
            .await
            .context("failed to install cert-manager")?;
        tracing::info!("cert-manager manifests applied");
        Ok(())
    }

    /// Apply GPU device-plugin manifests based on the GPUs that were detected.
    ///
    /// When `gpus` is empty this is a no-op.
    pub async fn install_gpu_operators(gpus: &[GpuInfo]) -> Result<()> {
        if gpus.is_empty() {
            tracing::info!("No GPUs detected — skipping GPU operator installation");
            return Ok(());
        }

        let vendors: HashSet<&str> =
            gpus.iter().map(|g| g.vendor.as_str()).collect();

        for vendor in vendors {
            match vendor {
                "nvidia" => {
                    tracing::info!("Installing NVIDIA GPU device plugin...");
                    Self::kubectl_apply(NVIDIA_MANIFEST)
                        .await
                        .context("failed to install NVIDIA device plugin")?;
                }
                "amd" => {
                    tracing::info!("Installing AMD GPU device plugin...");
                    Self::kubectl_apply(AMD_MANIFEST)
                        .await
                        .context("failed to install AMD device plugin")?;
                }
                other => {
                    tracing::warn!("Unknown GPU vendor '{other}' — skipping operator");
                }
            }
        }

        Ok(())
    }

    /// Apply RBAC manifests (namespace, service account, cluster role/binding).
    pub async fn install_rbac() -> Result<()> {
        tracing::info!("Applying RBAC manifests...");
        Self::kubectl_apply(RBAC_MANIFEST)
            .await
            .context("failed to apply RBAC manifests")?;
        tracing::info!("RBAC manifests applied");
        Ok(())
    }

    /// Check whether metrics-server is running.  This is informational only
    /// (k3s bundles metrics-server by default, but it can take a moment).
    pub async fn verify_metrics_server() -> Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args([
                "get",
                "deployment",
                "metrics-server",
                "-n",
                "kube-system",
                "-o",
                "jsonpath={.status.availableReplicas}",
            ])
            .output()
            .await
            .context("failed to run kubectl for metrics-server check")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let replicas = stdout.trim();
            if replicas.is_empty() || replicas == "0" {
                tracing::warn!(
                    "metrics-server is not available yet (may still be starting)"
                );
            } else {
                tracing::info!("metrics-server is running ({replicas} replica(s))");
            }
        } else {
            tracing::warn!(
                "could not query metrics-server (cluster may still be initialising)"
            );
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Pipe a YAML manifest into `kubectl apply -f -`.
    async fn kubectl_apply(manifest: &str) -> Result<()> {
        let mut child = tokio::process::Command::new("kubectl")
            .args(["apply", "-f", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("failed to spawn kubectl")?;

        // Write the manifest to stdin, then close the pipe
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(manifest.as_bytes())
                .await
                .context("failed to write manifest to kubectl stdin")?;
            // Dropping stdin closes the pipe, signalling EOF to kubectl
            drop(stdin);
        }

        let output = child
            .wait_with_output()
            .await
            .context("kubectl apply did not finish cleanly")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "kubectl apply failed\nstdout: {stdout}\nstderr: {stderr}"
            );
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
    fn test_manifests_are_not_empty() {
        assert!(!CERT_MANAGER_MANIFEST.is_empty());
        assert!(!RBAC_MANIFEST.is_empty());
        assert!(!NVIDIA_MANIFEST.is_empty());
        assert!(!AMD_MANIFEST.is_empty());
    }

    #[test]
    fn test_manifests_contain_k8s_kinds() {
        assert!(CERT_MANAGER_MANIFEST.contains("apiVersion"));
        assert!(CERT_MANAGER_MANIFEST.contains("CustomResourceDefinition"));
        assert!(NVIDIA_MANIFEST.contains("DaemonSet"));
        assert!(AMD_MANIFEST.contains("DaemonSet"));
        assert!(RBAC_MANIFEST.contains("Namespace"));
        assert!(RBAC_MANIFEST.contains("ClusterRole"));
    }
}
