use super::GpuVendor;
use anyhow::{Result, anyhow};
use ferrishost_core::GpuInfo;
use std::process::Command;

pub struct NvidiaGpu;

impl NvidiaGpu {
    pub fn new() -> Result<Self> {
        // Check if nvidia-smi exists
        Command::new("nvidia-smi")
            .arg("--version")
            .output()
            .map_err(|_| anyhow!("nvidia-smi not found"))?;
        Ok(NvidiaGpu)
    }

    fn get_smi_info() -> Result<Vec<(String, u64)>> {
        let output = Command::new("nvidia-smi")
            .arg("--query-gpu=name,memory.total")
            .arg("--format=csv,noheader")
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let mut gpus = Vec::new();

        for (_index, line) in stdout.lines().enumerate() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let memory_str = parts[1].trim().replace(" MiB", "");
                if let Ok(memory) = memory_str.parse::<u64>() {
                    gpus.push((name, memory));
                }
            }
        }

        Ok(gpus)
    }
}

impl GpuVendor for NvidiaGpu {
    fn detect(&self) -> Result<Vec<GpuInfo>> {
        let gpus = Self::get_smi_info()?;
        Ok(gpus
            .into_iter()
            .enumerate()
            .map(|(index, (name, memory))| GpuInfo {
                vendor: "nvidia".to_string(),
                name,
                memory_mb: memory,
                index: index as u32,
            })
            .collect())
    }

    fn prepare_host(&self) -> Result<()> {
        // Check if nvidia-container-toolkit is installed
        let output = Command::new("which").arg("nvidia-ctk").output();

        match output {
            Ok(out) if out.status.success() => {
                tracing::info!("nvidia-container-toolkit found — configuring containerd runtime");
                let result = Command::new("nvidia-ctk")
                    .args(["runtime", "configure", "--runtime=containerd"])
                    .output();

                match result {
                    Ok(r) if r.status.success() => {
                        tracing::info!(
                            "NVIDIA containerd runtime configured (may need containerd restart)"
                        );
                    }
                    Ok(r) => {
                        let stderr = String::from_utf8_lossy(&r.stderr);
                        tracing::warn!(
                            "nvidia-ctk configure failed: {stderr}"
                        );
                    }
                    Err(e) => {
                        tracing::warn!("failed to run nvidia-ctk: {e}");
                    }
                }
            }
            _ => {
                tracing::warn!(
                    "nvidia-container-toolkit not found. \
                     GPU workloads will not be able to use NVIDIA GPUs. \
                     Install it with:\n  \
                       sudo apt install nvidia-container-toolkit  # Debian/Ubuntu\n  \
                       sudo yum install nvidia-container-toolkit  # RHEL/Fedora\n  \
                     Then run: sudo nvidia-ctk runtime configure --runtime=containerd"
                );
            }
        }

        Ok(())
    }

    fn containerd_runtime_snippet(&self) -> Option<String> {
        Some(
            r#"
[plugins."io.containerd.grpc.v1.cri".containerd.runtimes.nvidia]
  runtime_engine = ""
  runtime_root = ""
  runtime_type = "io.containerd.runc.v2"
  [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.nvidia.options]
    BinaryName = "/usr/bin/nvidia-container-runtime"
"#
            .to_string(),
        )
    }

    fn cluster_manifests(&self) -> &'static str {
        include_str!("../../../manifests/nvidia-device-plugin.yaml")
    }

    fn resource_name(&self) -> &'static str {
        "nvidia.com/gpu"
    }
}
