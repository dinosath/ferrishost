use anyhow::{anyhow, Result};
use ferrishost_core::HostInfo;
use std::process::Command;

pub struct SystemInfo {
    pub os: String,
    pub kernel_version: String,
    pub arch: String,
    pub hostname: String,
}

impl SystemInfo {
    pub fn detect() -> Result<Self> {
        let os = Self::detect_os()?;
        let kernel_version = Self::detect_kernel_version()?;
        let arch = Self::detect_arch()?;
        let hostname = Self::detect_hostname()?;

        Ok(SystemInfo {
            os,
            kernel_version,
            arch,
            hostname,
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Verify it's Linux
        if self.os != "Linux" {
            return Err(anyhow!("FerrisHost only supports Linux (detected: {})", self.os));
        }

        // Verify supported architecture
        match self.arch.as_str() {
            "x86_64" | "aarch64" => {},
            arch => return Err(anyhow!("Unsupported architecture: {} (only x86_64/aarch64 supported)", arch)),
        }

        Ok(())
    }

    fn detect_os() -> Result<String> {
        let output = Command::new("uname")
            .arg("-s")
            .output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn detect_kernel_version() -> Result<String> {
        let output = Command::new("uname")
            .arg("-r")
            .output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn detect_arch() -> Result<String> {
        let output = Command::new("uname")
            .arg("-m")
            .output()?;
        let arch = String::from_utf8(output.stdout)?.trim().to_string();
        // Normalize to standard names
        Ok(match arch.as_str() {
            "x86_64" => "x86_64".to_string(),
            "aarch64" | "arm64" => "aarch64".to_string(),
            other => other.to_string(),
        })
    }

    fn detect_hostname() -> Result<String> {
        let output = Command::new("hostname")
            .output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    pub fn to_host_info(&self) -> HostInfo {
        HostInfo {
            os: self.os.clone(),
            kernel_version: self.kernel_version.clone(),
            arch: self.arch.clone(),
            hostname: self.hostname.clone(),
        }
    }
}
