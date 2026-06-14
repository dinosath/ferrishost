use ferrishost_core::{
    ClusterStatus, GpuInfo, GpuStatus, HostInfo, ModuleDescriptor, NAMESPACE, NodeStatus,
    SetupState, TlsMode, WEB_SERVICE_NAME,
};

// ---------------------------------------------------------------------------
// GpuInfo / GpuStatus
// ---------------------------------------------------------------------------

#[test]
fn gpu_info_serialise_roundtrip() {
    let info = GpuInfo {
        vendor: "nvidia".to_string(),
        name: "RTX 4090".to_string(),
        memory_mb: 24576,
        index: 0,
    };
    let json = serde_json::to_string(&info).expect("serialise");
    let back: GpuInfo = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(back.vendor, info.vendor);
    assert_eq!(back.name, info.name);
    assert_eq!(back.memory_mb, info.memory_mb);
    assert_eq!(back.index, info.index);
}

#[test]
fn gpu_status_empty_defaults() {
    let status = GpuStatus {
        detected: vec![],
        nvidia_available: false,
        amd_available: false,
    };
    assert!(status.detected.is_empty());
    assert!(!status.nvidia_available);
    assert!(!status.amd_available);
}

#[test]
fn gpu_status_with_nvidia_gpu() {
    let status = GpuStatus {
        detected: vec![GpuInfo {
            vendor: "nvidia".to_string(),
            name: "A100".to_string(),
            memory_mb: 40960,
            index: 0,
        }],
        nvidia_available: true,
        amd_available: false,
    };
    let json = serde_json::to_string(&status).expect("serialise");
    let back: GpuStatus = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(back.detected.len(), 1);
    assert!(back.nvidia_available);
    assert!(!back.amd_available);
}

// ---------------------------------------------------------------------------
// HostInfo
// ---------------------------------------------------------------------------

#[test]
fn host_info_serialise_roundtrip() {
    let info = HostInfo {
        os: "Linux".to_string(),
        kernel_version: "6.1.0".to_string(),
        arch: "x86_64".to_string(),
        hostname: "myhost".to_string(),
    };
    let json = serde_json::to_string(&info).expect("serialise");
    let back: HostInfo = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(back.os, "Linux");
    assert_eq!(back.arch, "x86_64");
    assert_eq!(back.hostname, "myhost");
}

// ---------------------------------------------------------------------------
// NodeStatus / ClusterStatus
// ---------------------------------------------------------------------------

#[test]
fn node_status_ready_flag() {
    let node = NodeStatus {
        name: "node-1".to_string(),
        ready: true,
        cpu_millis: 4000,
        memory_mb: 8192,
    };
    assert!(node.ready);
    assert_eq!(node.cpu_millis, 4000);
}

#[test]
fn cluster_status_serialise_roundtrip() {
    let cluster = ClusterStatus {
        k3s_installed: true,
        k3s_version: Some("v1.29.0+k3s1".to_string()),
        nodes: vec![NodeStatus {
            name: "server".to_string(),
            ready: true,
            cpu_millis: 2000,
            memory_mb: 4096,
        }],
        all_nodes_ready: true,
    };
    let json = serde_json::to_string(&cluster).expect("serialise");
    let back: ClusterStatus = serde_json::from_str(&json).expect("deserialise");
    assert!(back.k3s_installed);
    assert_eq!(back.k3s_version.as_deref(), Some("v1.29.0+k3s1"));
    assert_eq!(back.nodes.len(), 1);
    assert!(back.all_nodes_ready);
}

#[test]
fn cluster_status_no_k3s() {
    let cluster = ClusterStatus {
        k3s_installed: false,
        k3s_version: None,
        nodes: vec![],
        all_nodes_ready: false,
    };
    assert!(!cluster.k3s_installed);
    assert!(cluster.k3s_version.is_none());
}

// ---------------------------------------------------------------------------
// ModuleDescriptor
// ---------------------------------------------------------------------------

#[test]
fn module_descriptor_fields() {
    let module = ModuleDescriptor {
        id: "headscale".to_string(),
        name: "Headscale".to_string(),
        description: "Private mesh VPN".to_string(),
        category: "vpn".to_string(),
        version: "0.22.0".to_string(),
        installed: false,
    };
    assert_eq!(module.id, "headscale");
    assert_eq!(module.category, "vpn");
    assert!(!module.installed);
}

#[test]
fn module_descriptor_serialise_roundtrip() {
    let module = ModuleDescriptor {
        id: "seafile".to_string(),
        name: "Seafile".to_string(),
        description: "File sync".to_string(),
        category: "app".to_string(),
        version: "9.0.0".to_string(),
        installed: true,
    };
    let json = serde_json::to_string(&module).expect("serialise");
    let back: ModuleDescriptor = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(back.id, "seafile");
    assert!(back.installed);
}

// ---------------------------------------------------------------------------
// SetupState / TlsMode
// ---------------------------------------------------------------------------

#[test]
fn setup_state_defaults_incomplete() {
    let state = SetupState {
        domain: None,
        tls_mode: TlsMode::SelfSigned,
        admin_username: None,
        admin_password_hash: None,
        timezone: None,
        completed: false,
    };
    assert!(!state.completed);
    assert!(state.domain.is_none());
}

#[test]
fn setup_state_completed() {
    let state = SetupState {
        domain: Some("home.example.com".to_string()),
        tls_mode: TlsMode::LetsEncrypt,
        admin_username: Some("admin".to_string()),
        admin_password_hash: Some("$argon2id$...".to_string()),
        timezone: Some("UTC".to_string()),
        completed: true,
    };
    assert!(state.completed);
    assert_eq!(state.domain.as_deref(), Some("home.example.com"));
}

#[test]
fn tls_mode_serialises_lowercase() {
    let json = serde_json::to_string(&TlsMode::LetsEncrypt).expect("serialise");
    assert_eq!(json, r#""letsencrypt""#);
    let json2 = serde_json::to_string(&TlsMode::SelfSigned).expect("serialise");
    assert_eq!(json2, r#""selfsigned""#);
}

#[test]
fn setup_state_roundtrip() {
    let original = SetupState {
        domain: Some("cloud.local".to_string()),
        tls_mode: TlsMode::LetsEncrypt,
        admin_username: Some("root".to_string()),
        admin_password_hash: None,
        timezone: Some("Europe/London".to_string()),
        completed: false,
    };
    let json = serde_json::to_string(&original).expect("serialise");
    let back: SetupState = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(back.domain, original.domain);
    assert_eq!(back.timezone, original.timezone);
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn constants_have_expected_values() {
    assert_eq!(NAMESPACE, "ferrishost");
    assert_eq!(WEB_SERVICE_NAME, "ferrishost-web");
}
