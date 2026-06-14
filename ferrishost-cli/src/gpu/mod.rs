pub mod amd;
pub mod nvidia;

use anyhow::Result;
use ferrishost_core::GpuInfo;

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
}
