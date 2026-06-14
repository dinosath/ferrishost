use super::GpuVendor;
use anyhow::{Result, anyhow};
use ferrishost_core::GpuInfo;
use std::fs;
use std::path::Path;

pub struct AmdGpu;

impl AmdGpu {
    pub fn new() -> Result<Self> {
        // Check for AMD GPU via /dev/kfd
        if !Path::new("/dev/kfd").exists() {
            return Err(anyhow!("No AMD GPU detected (/dev/kfd not found)"));
        }
        Ok(AmdGpu)
    }

    fn detect_devices() -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();
        let drm_path = Path::new("/sys/class/drm");

        if !drm_path.exists() {
            return Ok(gpus);
        }

        for entry in fs::read_dir(drm_path)? {
            let entry = entry?;
            let path = entry.path();

            // Look for card* devices
            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.starts_with("card") {
                        // Check if it's an AMD device (vendor ID 0x1002)
                        let vendor_path = path.join("device/vendor");
                        if let Ok(vendor_id) = fs::read_to_string(vendor_path) {
                            if vendor_id.trim() == "0x1002" {
                                let index = gpus.len() as u32;
                                gpus.push(GpuInfo {
                                    vendor: "amd".to_string(),
                                    name: format!("AMD Radeon GPU {}", index),
                                    memory_mb: 0, // AMD detection doesn't easily give us this
                                    index,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(gpus)
    }
}

impl GpuVendor for AmdGpu {
    fn detect(&self) -> Result<Vec<GpuInfo>> {
        Self::detect_devices()
    }

    fn prepare_host(&self) -> Result<()> {
        // Ensure amdgpu kernel module is loaded
        // Check for /dev/dri accessibility
        tracing::info!("Checking AMD GPU host configuration...");

        // Verify /dev/kfd exists
        if !Path::new("/dev/kfd").exists() {
            return Err(anyhow!("/dev/kfd not found"));
        }

        // Check for /dev/dri devices
        let dri_path = Path::new("/dev/dri");
        if !dri_path.exists() {
            return Err(anyhow!("/dev/dri not found"));
        }

        Ok(())
    }

    fn containerd_runtime_snippet(&self) -> Option<String> {
        // AMD doesn't need a custom runtime class, but we define one for consistency
        None
    }

    fn cluster_manifests(&self) -> &'static str {
        include_str!("../../../manifests/amd-device-plugin.yaml")
    }

    fn resource_name(&self) -> &'static str {
        "amd.com/gpu"
    }
}
