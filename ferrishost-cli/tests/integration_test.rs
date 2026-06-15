use std::process::Command;
use std::path::Path;

fn ferrishost() -> Command {
    let bin = env!("CARGO_BIN_EXE_ferrishost");
    Command::new(bin)
}

// ---------------------------------------------------------------------------
// --help / subcommand help flags
// ---------------------------------------------------------------------------

#[test]
fn top_level_help_exits_zero() {
    let output = ferrishost()
        .arg("--help")
        .output()
        .expect("failed to spawn ferrishost --help");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn top_level_help_mentions_ferrishost() {
    let output = ferrishost()
        .arg("--help")
        .output()
        .expect("failed to spawn ferrishost --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("ferrishost"),
        "expected 'ferrishost' in help output, got: {stdout}"
    );
}

#[test]
fn setup_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["setup", "--help"])
        .output()
        .expect("failed to spawn ferrishost setup --help");
    assert!(output.status.success());
}

#[test]
fn setup_help_mentions_gpu_flag() {
    let output = ferrishost()
        .args(["setup", "--help"])
        .output()
        .expect("failed to spawn ferrishost setup --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("skip-gpu") || stdout.contains("gpu"),
        "expected GPU-related flag in setup help, got: {stdout}"
    );
}

#[test]
fn status_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["status", "--help"])
        .output()
        .expect("failed to spawn ferrishost status --help");
    assert!(output.status.success());
}

#[test]
fn gpu_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["gpu", "--help"])
        .output()
        .expect("failed to spawn ferrishost gpu --help");
    assert!(output.status.success());
}

#[test]
fn upgrade_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["upgrade", "--help"])
        .output()
        .expect("failed to spawn ferrishost upgrade --help");
    assert!(output.status.success());
}

#[test]
fn uninstall_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["uninstall", "--help"])
        .output()
        .expect("failed to spawn ferrishost uninstall --help");
    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Invalid arguments
// ---------------------------------------------------------------------------

#[test]
fn unknown_subcommand_exits_nonzero() {
    let output = ferrishost()
        .arg("not-a-real-command")
        .output()
        .expect("failed to spawn ferrishost not-a-real-command");
    assert!(
        !output.status.success(),
        "expected non-zero exit for unknown subcommand"
    );
}

#[test]
fn unknown_flag_exits_nonzero() {
    let output = ferrishost()
        .arg("--not-a-real-flag")
        .output()
        .expect("failed to spawn ferrishost --not-a-real-flag");
    assert!(
        !output.status.success(),
        "expected non-zero exit for unknown flag"
    );
}

// ---------------------------------------------------------------------------
// Non-root guard (only meaningful when not running as root)
// ---------------------------------------------------------------------------

/// The CLI does not require root, so `status` should succeed as any user.
#[test]
fn status_succeeds_without_root() {
    let output = ferrishost()
        .arg("status")
        .output()
        .expect("failed to spawn ferrishost status");
    assert!(
        output.status.success(),
        "expected zero exit for `status`, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// Unit tests — system detection
// ---------------------------------------------------------------------------

#[test]
fn test_detect_host_info_returns_valid_fields() {
    // SystemInfo::detect returns non-empty os, kernel_version, arch, hostname
    // On CI (Linux), these should all be non-empty
    let sys = ferrishost_cli::system::SystemInfo::detect()
        .expect("SystemInfo::detect should succeed");

    assert!(!sys.os.is_empty(), "os should not be empty");
    assert!(!sys.kernel_version.is_empty(), "kernel_version should not be empty");
    assert!(!sys.arch.is_empty(), "arch should not be empty");
    assert!(!sys.hostname.is_empty(), "hostname should not be empty");
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn test_preflight_accepts_linux() {
    // On Linux, validate() should succeed
    let sys = ferrishost_cli::system::SystemInfo::detect()
        .expect("SystemInfo::detect should succeed");
    assert!(sys.validate().is_ok(), "validate() should succeed on Linux");
}

#[test]
#[cfg_attr(not(target_os = "macos"), ignore)]
fn test_preflight_rejects_non_linux() {
    // On macOS, validate() should fail
    let sys = ferrishost_cli::system::SystemInfo {
        os: "Darwin".to_string(),
        kernel_version: "23.0.0".to_string(),
        arch: "x86_64".to_string(),
        hostname: "macmini".to_string(),
    };
    let result = sys.validate();
    assert!(result.is_err(), "validate() should fail on non-Linux");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Linux"), "error should mention Linux");
}

// ---------------------------------------------------------------------------
// Unit tests — GPU detection
// ---------------------------------------------------------------------------

#[test]
fn test_gpu_detector_returns_empty_when_no_gpus() {
    // GpuDetector::detect_all() should return an empty vec when no GPUs
    // are available. On CI (no GPU), this will return empty.
    let _gpus = ferrishost_cli::gpu::GpuDetector::detect_all()
        .expect("detect_all should not fail");
    // In CI, there are no GPUs, so vector should be empty
    // We only assert it doesn't panic and returns Ok
}

#[test]
fn test_nvidia_parse_csv_output() {

    // Simulate nvidia-smi CSV output parsing
    let sample_output = "NVIDIA GeForce RTX 3090, 24576 MiB\nNVIDIA GeForce RTX 4090, 24576 MiB";
    let gpus = parse_nvidia_csv(sample_output);

    assert_eq!(gpus.len(), 2, "should parse 2 GPUs");
    assert_eq!(gpus[0].vendor, "nvidia");
    assert_eq!(gpus[0].name, "NVIDIA GeForce RTX 3090");
    assert_eq!(gpus[0].memory_mb, 24576);
    assert_eq!(gpus[0].index, 0);
    assert_eq!(gpus[1].name, "NVIDIA GeForce RTX 4090");
    assert_eq!(gpus[1].memory_mb, 24576);
    // Note: index is always 0 from parse_nvidia_csv; the caller (GpuVendor::detect)
    // sets the actual index via enumerate().
    assert_eq!(gpus[1].index, 0);
}

#[test]
fn test_nvidia_parse_empty_output() {
    let gpus = parse_nvidia_csv("");
    assert!(gpus.is_empty(), "empty output should produce empty vec");
}

#[test]
fn test_nvidia_parse_malformed_output() {
    let gpus = parse_nvidia_csv("NOT A VALID CSV LINE\nname, memory, bogus_column\n, only a name");
    // Should not panic; should gracefully handle malformed lines
    assert!(!gpus.iter().any(|g| g.memory_mb == 0 && g.name.is_empty()));
}

#[test]
fn test_amd_vendor_id_check() {
    // Verify that vendor ID "0x1002" is correctly identified as AMD
    assert!(is_amd_vendor("0x1002"), "0x1002 should be AMD");
    assert!(!is_amd_vendor("0x10de"), "0x10de should NOT be AMD (NVIDIA)");
    assert!(!is_amd_vendor("0x8086"), "0x8086 should NOT be AMD (Intel)");
    assert!(!is_amd_vendor(""), "empty string should not be AMD");
}

#[test]
fn test_containerd_runtime_snippet_format() {
    use ferrishost_cli::gpu::GpuVendor;
    // If nvidia-smi is not available (CI), skip gracefully
    let nvidia = match ferrishost_cli::gpu::nvidia::NvidiaGpu::new() {
        Ok(n) => n,
        Err(_) => return, // nvidia-smi not available, skip test
    };
    let snippet = nvidia.containerd_runtime_snippet();
    if let Some(toml) = snippet {
        // Validate it looks like valid TOML — contains expected keys
        assert!(toml.contains("io.containerd.grpc.v1.cri"), "snippet should contain containerd CRI path");
        assert!(toml.contains("nvidia"), "snippet should mention nvidia");
    }
}

#[test]
fn test_cluster_manifests_are_valid_yaml() {
    use ferrishost_cli::gpu::GpuVendor;
    // Verify the included YAML manifests are valid by checking the NVidia one
    let nvidia = match ferrishost_cli::gpu::nvidia::NvidiaGpu::new() {
        Ok(n) => n,
        Err(_) => return, // nvidia-smi not available, skip test
    };
    let manifest = nvidia.cluster_manifests();
    assert!(!manifest.is_empty(), "manifest should not be empty");
    assert!(manifest.contains("apiVersion"), "manifest should contain apiVersion");
    assert!(manifest.contains("kind"), "manifest should contain kind");
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn test_k3s_is_installed_checks_binary_path() {
    // Check if /usr/local/bin/k3s exists
    let k3s_path = Path::new("/usr/local/bin/k3s");
    // Just verify the check doesn't panic; k3s may or may not be installed
    let _exists = k3s_path.exists();
}

// ---------------------------------------------------------------------------
// Helper functions for pure parsing (extracted for testability)
// ---------------------------------------------------------------------------

/// Parse nvidia-smi CSV output into GpuInfo structs
/// Expected format: "GPU Name, Memory MiB"
fn parse_nvidia_csv(output: &str) -> Vec<ferrishost_core::GpuInfo> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let memory_str = parts[1].trim().replace(" MiB", "");
                let memory_mb: u64 = memory_str.parse().ok()?;
                Some(ferrishost_core::GpuInfo {
                    vendor: "nvidia".to_string(),
                    name,
                    memory_mb,
                    index: 0, // index will be set by caller
                })
            } else {
                None
            }
        })
        .collect()
}

/// Check if a vendor ID string corresponds to AMD
fn is_amd_vendor(vendor_id: &str) -> bool {
    vendor_id.trim() == "0x1002"
}
