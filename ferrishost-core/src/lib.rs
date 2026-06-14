use serde::{Deserialize, Serialize};

/// Information about a detected GPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,        // "nvidia" or "amd"
    pub name: String,
    pub memory_mb: u64,
    pub index: u32,
}

/// Aggregated GPU status from all vendors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub detected: Vec<GpuInfo>,
    pub nvidia_available: bool,
    pub amd_available: bool,
}

/// Host OS and architecture information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub os: String,            // "Linux"
    pub kernel_version: String,
    pub arch: String,          // "x86_64" or "aarch64"
    pub hostname: String,
}

/// Kubernetes node status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub name: String,
    pub ready: bool,
    pub cpu_millis: u32,
    pub memory_mb: u32,
}

/// Overall cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub k3s_installed: bool,
    pub k3s_version: Option<String>,
    pub nodes: Vec<NodeStatus>,
    pub all_nodes_ready: bool,
}

/// Descriptor for an optional module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,       // "storage", "auth", "vpn", "app", etc.
    pub version: String,
    pub installed: bool,
}

/// Setup wizard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupState {
    pub domain: Option<String>,
    pub tls_mode: TlsMode,
    pub admin_username: Option<String>,
    pub admin_password_hash: Option<String>,
    pub timezone: Option<String>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TlsMode {
    SelfSigned,
    LetsEncrypt,
}

// Constants
pub const NAMESPACE: &str = "ferrishost";
pub const WEB_SERVICE_NAME: &str = "ferrishost-web";
