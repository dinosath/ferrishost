pub mod amd;
pub mod nvidia;

use anyhow::{Context, Result};
use ferrishost_core::GpuInfo;
use std::collections::HashSet;

pub trait GpuVendor {
    fn detect(&self) -> Result<Vec<GpuInfo>>;
    fn prepare_host(&self) -> Result<()>;
    fn containerd_runtime_snippet(&self) -> Option<String>;
    fn cluster_manifests(&self) -> &'static str;
    fn resource_name(&self) -> &'static str;
}

pub struct GpuDetector;

impl GpuDetector {
    pub fn detect_all() -> Result<Vec<GpuInfo>> {
        let mut detected = Vec::new();

        // Try NVIDIA
        if let Ok(nvidia_detector) = nvidia::NvidiaGpu::new() {
            if let Ok(gpus) = nvidia_detector.detect() {
                detected.extend(gpus);
            }
        }

        // Try AMD
        if let Ok(amd_detector) = amd::AmdGpu::new() {
            if let Ok(gpus) = amd_detector.detect() {
                detected.extend(gpus);
            }
        }

        Ok(detected)
    }

    /// Run host-preparation steps for each unique GPU vendor found in `gpus`.
    ///
    /// Returns an optional containerd runtime snippet (only NVIDIA provides one)
    /// that should be applied to the k3s containerd configuration.
    pub fn prepare_and_get_config(gpus: &[GpuInfo]) -> Result<Option<String>> {
        let vendors: HashSet<&str> =
            gpus.iter().map(|g| g.vendor.as_str()).collect();
        let mut containerd_snippet = None;

        for vendor in vendors {
            match vendor {
                "nvidia" => {
                    let detector = nvidia::NvidiaGpu::new().context(
                        "NVIDIA GPU reported but nvidia-smi not found",
                    )?;
                    detector.prepare_host()?;
                    if containerd_snippet.is_none() {
                        containerd_snippet = detector.containerd_runtime_snippet();
                    }
                }
                "amd" => {
                    let detector = amd::AmdGpu::new().context(
                        "AMD GPU reported but /dev/kfd not found",
                    )?;
                    detector.prepare_host()?;
                }
                other => {
                    tracing::warn!(
                        "Unknown GPU vendor '{other}', skipping host preparation"
                    );
                }
            }
        }

        Ok(containerd_snippet)
    }
}
