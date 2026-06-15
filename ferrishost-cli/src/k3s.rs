use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

/// k3s binary location after installation.
const K3S_BINARY: &str = "/usr/local/bin/k3s";
/// Official k3s install script URL.
const K3S_INSTALL_URL: &str = "https://get.k3s.io";
/// Source kubeconfig created by k3s.
const KUBECONFIG_SRC: &str = "/etc/rancher/k3s/k3s.yaml";
/// How long (seconds) to wait for the node to become ready.
const READY_TIMEOUT_SECS: u64 = 300;

pub struct K3s;

impl K3s {
    /// Check whether the k3s binary exists on the host.
    pub fn is_installed() -> bool {
        Path::new(K3S_BINARY).exists()
    }

    /// Download and run the k3s install script.
    ///
    /// If k3s is already installed this is a no-op.
    pub async fn install(disable_traefik: bool, offline: bool) -> Result<()> {
        if Self::is_installed() {
            tracing::info!("k3s is already installed at {K3S_BINARY}");
            return Ok(());
        }

        if offline {
            anyhow::bail!(
                "k3s is not installed and --offline is set; \
                 cannot download the installer from {K3S_INSTALL_URL}"
            );
        }

        tracing::info!("Downloading k3s install script from {K3S_INSTALL_URL}...");

        // Download the install script first so we can report curl errors clearly.
        let curl_output = tokio::process::Command::new("curl")
            .args(["-sfL", K3S_INSTALL_URL])
            .output()
            .await
            .context("failed to spawn curl — is curl installed?")?;

        if !curl_output.status.success() {
            let stderr = String::from_utf8_lossy(&curl_output.stderr);
            anyhow::bail!(
                "failed to download k3s install script (curl exited {})\n{}",
                curl_output.status
                    .code()
                    .map_or("unknown".into(), |c| c.to_string()),
                stderr,
            );
        }

        let script_bytes = curl_output.stdout;
        tracing::info!(
            "Downloaded {} bytes. Running install script...",
            script_bytes.len()
        );

        // Pipe the script into `sh`.  We pass INSTALL_K3S_EXEC to control
        // optional features.
        let mut child = tokio::process::Command::new("sh")
            .arg("-s") // read script from stdin
            .env(
                "INSTALL_K3S_EXEC",
                if disable_traefik { "--disable traefik" } else { "" },
            )
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .context("failed to spawn sh")?;

        // Write the script bytes and close stdin to signal EOF.
        if let Some(mut stdin) = child.stdin.take() {
            tokio::io::AsyncWriteExt::write_all(&mut stdin, &script_bytes)
                .await
                .context("failed to write install script to sh stdin")?;
            drop(stdin);
        }

        let status = child
            .wait()
            .await
            .context("k3s install script did not finish cleanly")?;

        if !status.success() {
            anyhow::bail!(
                "k3s install script failed (exit {})",
                status.code().map_or("unknown".into(), |c| c.to_string()),
            );
        }

        tracing::info!("k3s installed successfully");
        Ok(())
    }

    /// Poll `kubectl get nodes` until the node reports Ready, or until
    /// `timeout_secs` elapses.
    pub async fn wait_for_ready_with_timeout(
        timeout_secs: u64,
    ) -> Result<()> {
        let start = Instant::now();

        tracing::info!("Waiting for k3s node to become ready...");

        loop {
            if start.elapsed().as_secs() > timeout_secs {
                anyhow::bail!(
                    "timed out after {timeout_secs}s waiting for node to become ready"
                );
            }

            let output = tokio::process::Command::new("kubectl")
                .args([
                    "--kubeconfig", KUBECONFIG_SRC,
                    "get",
                    "nodes",
                    "-o",
                    "jsonpath={.items[*].status.conditions[?(@.type=='Ready')].status}",
                ])
                .output()
                .await;

            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if stdout.contains("True") {
                        tracing::info!("k3s node is ready");
                        return Ok(());
                    }
                }
                Ok(_) => {
                    // kubectl ran but returned non-zero (node not ready yet
                    // or API server not listening).
                }
                Err(e) => {
                    // kubectl binary not available yet or similar.
                    tracing::debug!("kubectl check failed (retrying): {e}");
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    /// Convienence wrapper that uses the default timeout (120 s).
    pub async fn wait_for_ready() -> Result<()> {
        Self::wait_for_ready_with_timeout(READY_TIMEOUT_SECS).await
    }

    /// Copy the k3s-generated kubeconfig to `~/.kube/config` so that
    /// `kubectl` works without `--kubeconfig`.
    ///
    /// If the target already exists or the source is not readable (e.g. when
    /// running without sudo), a non-fatal warning is logged instead.
    pub fn copy_kubeconfig() -> Result<()> {
        let home = std::env::var("HOME")
            .unwrap_or_else(|_| "/root".to_string());
        let kube_dir = Path::new(&home).join(".kube");
        let target = kube_dir.join("config");

        if target.exists() {
            tracing::info!("{} already exists — skipping copy", target.display());
            return Ok(());
        }

        if !Path::new(KUBECONFIG_SRC).exists() {
            tracing::warn!(
                "k3s kubeconfig not found at {KUBECONFIG_SRC} — has k3s been started?"
            );
            return Ok(());
        }

        std::fs::create_dir_all(&kube_dir)
            .context("failed to create ~/.kube directory")?;

        match std::fs::copy(KUBECONFIG_SRC, &target) {
            Ok(_) => {
                // Restrict permissions on the copied kubeconfig.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &target,
                        std::fs::Permissions::from_mode(0o600),
                    );
                }
                tracing::info!("kubeconfig copied to {}", target.display());
                Ok(())
            }
            Err(e) => {
                // Non-fatal — subsequent kubectl calls can use --kubeconfig directly.
                tracing::warn!(
                    "Could not copy kubeconfig to {path}: {e}.\n  \
                     Run: sudo cp {KUBECONFIG_SRC} {path}",
                    path = target.display(),
                );
                Ok(())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_installed_does_not_panic() {
        // On a development machine k3s may or may not be present; just check
        // that the function runs without panicking.
        let _ = K3s::is_installed();
    }
}
