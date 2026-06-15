use anyhow::Context;
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::Path;
use tracing_subscriber;

use ferrishost_cli::{deploy_web, gpu, k3s, operators, system};
use system::SystemInfo;

#[derive(Parser)]
#[command(name = "ferrishost")]
#[command(about = "FerrisHost — lightweight personal cloud bootstrapper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(global = true, short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Command {
    /// Bootstrap a new FerrisHost installation (default)
    Setup {
        /// Skip GPU detection and configuration
        #[arg(long)]
        skip_gpu: bool,

        /// Force GPU vendor (nvidia, amd, auto)
        #[arg(long)]
        gpu_vendor: Option<String>,

        /// Disable Traefik ingress controller
        #[arg(long)]
        disable_traefik: bool,

        /// Port to expose ferrishost-web on
        #[arg(long, default_value = "443")]
        web_port: u16,

        /// Don't automatically open a browser
        #[arg(long)]
        no_browser: bool,

        /// Skip steps requiring network access
        #[arg(long)]
        offline: bool,
    },

    /// Check host / k3s / GPU / ferrishost-web status
    Status,

    /// Detect and report GPU information
    Gpu {
        /// Only detect; don't prepare or install
        #[arg(long)]
        detect_only: bool,
    },

    /// Upgrade FerrisHost to current versions
    Upgrade,

    /// Uninstall FerrisHost
    Uninstall {
        /// Also uninstall k3s
        #[arg(long)]
        also_k3s: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter_level)
        .init();

    // Route to the appropriate command
    let command = cli.command.unwrap_or(Command::Setup {
        skip_gpu: false,
        gpu_vendor: None,
        disable_traefik: false,
        web_port: 443,
        no_browser: false,
        offline: false,
    });

    match command {
        Command::Setup {
            skip_gpu,
            gpu_vendor,
            disable_traefik,
            web_port,
            no_browser,
            offline,
        } => {
            setup(
                skip_gpu,
                gpu_vendor,
                disable_traefik,
                web_port,
                no_browser,
                offline,
            )
            .await?;
        }
        Command::Status => {
            status().await?;
        }
        Command::Gpu { detect_only } => {
            gpu_command(detect_only).await?;
        }
        Command::Upgrade => {
            upgrade().await?;
        }
        Command::Uninstall { also_k3s } => {
            uninstall(also_k3s).await?;
        }
    }

    Ok(())
}

async fn setup(
    skip_gpu: bool,
    gpu_vendor: Option<String>,
    disable_traefik: bool,
    _web_port: u16,
    _no_browser: bool,
    offline: bool,
) -> anyhow::Result<()> {
    tracing::info!("Starting FerrisHost setup...");

    // ------------------------------------------------------------------
    // Step 1: Preflight checks
    // ------------------------------------------------------------------
    tracing::info!("[1/6] Running preflight checks...");
    let sys = SystemInfo::detect()?;
    sys.validate()?;
    tracing::info!(
        "  OS: {} {}, Arch: {}",
        sys.os,
        sys.kernel_version,
        sys.arch
    );
    tracing::info!("  Hostname: {}", sys.hostname);

    // ------------------------------------------------------------------
    // Step 2: GPU detection
    // ------------------------------------------------------------------
    let mut gpus = Vec::new();
    let mut containerd_snippet: Option<String> = None;

    if !skip_gpu {
        tracing::info!("[2/6] Detecting GPU configuration...");

        gpus = gpu::GpuDetector::detect_all()?;

        if gpus.is_empty() {
            tracing::info!("  No GPUs detected");
        } else {
            for gpu in &gpus {
                tracing::info!(
                    "  Detected: {} {} ({} MB)",
                    gpu.vendor,
                    gpu.name,
                    gpu.memory_mb
                );
            }

            // If the user forced a vendor, validate
            if let Some(ref forced) = gpu_vendor {
                let has_vendor = gpus.iter().any(|g| g.vendor == *forced);
                if !has_vendor {
                    tracing::warn!(
                        "Forced GPU vendor '{forced}' was requested but not detected; \
                         continuing with detected GPUs"
                    );
                }
            }

            // Run vendor-specific host preparation
            containerd_snippet =
                gpu::GpuDetector::prepare_and_get_config(&gpus)?;
        }
    } else {
        tracing::info!("[2/6] Skipping GPU detection (--skip-gpu)");
    }

    // ------------------------------------------------------------------
    // Step 3: Install k3s
    // ------------------------------------------------------------------
    tracing::info!(
        "[3/6] Installing k3s (disable_traefik={disable_traefik}, offline={offline})..."
    );
    k3s::K3s::install(disable_traefik, offline).await?;
    k3s::K3s::copy_kubeconfig()?;
    k3s::K3s::wait_for_ready().await?;

    // If NVIDIA GPU: apply containerd runtime config to k3s
    if let Some(snippet) = &containerd_snippet {
        let k3s_containerd_dir =
            Path::new("/var/lib/rancher/k3s/agent/etc/containerd");
        if k3s_containerd_dir.exists() {
            let config_tmpl = k3s_containerd_dir.join("config.toml.tmpl");
            tracing::info!(
                "  Writing NVIDIA containerd runtime snippet to {}",
                config_tmpl.display()
            );
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&config_tmpl)
                .context("failed to open k3s containerd config template")?;
            file.write_all(snippet.as_bytes())
                .context("failed to write containerd runtime snippet")?;
            tracing::info!(
                "  NOTE: Restart k3s to apply: sudo systemctl restart k3s"
            );
        } else {
            tracing::warn!(
                "  k3s containerd directory not found; skipping NVIDIA runtime config"
            );
        }
    }

    // ------------------------------------------------------------------
    // Step 4: Install core operators
    // ------------------------------------------------------------------
    tracing::info!("[4/6] Installing core operators...");
    operators::Operators::install_cert_manager().await?;
    operators::Operators::install_gpu_operators(&gpus).await?;
    operators::Operators::install_rbac().await?;
    operators::Operators::verify_metrics_server().await?;

    // ------------------------------------------------------------------
    // Step 5: Deploy ferrishost-web
    // ------------------------------------------------------------------
    tracing::info!("[5/6] Deploying ferrishost-web...");
    let ingress_host = format!("{}.homelab.local", sys.hostname);
    let deployer = deploy_web::WebDeployer::new(&ingress_host);
    deployer.deploy().await?;
    let web_url = format!("https://{}", ingress_host);

    // ------------------------------------------------------------------
    // Step 6: Hand off
    // ------------------------------------------------------------------
    tracing::info!("[6/6] Setup complete!");
    tracing::info!("  Web dashboard: {web_url}");
    if !gpus.is_empty() {
        tracing::info!("  GPU support: {} GPU(s) configured", gpus.len());
    }

    Ok(())
}

async fn status() -> anyhow::Result<()> {
    println!("FerrisHost Status");
    println!("================\n");

    // Host info
    match SystemInfo::detect() {
        Ok(sys) => {
            println!(
                "Host:   {} / {} {} ({})",
                sys.hostname, sys.os, sys.kernel_version, sys.arch
            );
        }
        Err(e) => {
            println!("Host:   detection failed — {e}");
        }
    }

    // k3s
    if k3s::K3s::is_installed() {
        println!("k3s:   installed at /usr/local/bin/k3s");
    } else {
        println!("k3s:   not installed");
    }

    // GPU
    match gpu::GpuDetector::detect_all() {
        Ok(gpus) => {
            if gpus.is_empty() {
                println!("GPU:   none detected");
            } else {
                println!("GPU:   {} detected", gpus.len());
                for gpu in &gpus {
                    println!(
                        "         [{}] {} {} ({} MB)",
                        gpu.index, gpu.vendor, gpu.name, gpu.memory_mb
                    );
                }
            }
        }
        Err(e) => {
            println!("GPU:   detection error — {e}");
        }
    }

    Ok(())
}

async fn gpu_command(detect_only: bool) -> anyhow::Result<()> {
    println!("GPU Detection");
    println!("=============\n");

    let gpus = gpu::GpuDetector::detect_all()?;

    if gpus.is_empty() {
        println!("No GPUs detected.");
        return Ok(());
    }

    println!("Detected {} GPU(s):\n", gpus.len());
    for gpu in &gpus {
        println!(
            "  [{}] {} {} ({} MB)",
            gpu.index, gpu.vendor, gpu.name, gpu.memory_mb
        );
    }

    if !detect_only {
        println!("\nRunning host preparation...");
        gpu::GpuDetector::prepare_and_get_config(&gpus)?;
        println!("Host preparation complete.");
    }

    Ok(())
}

async fn upgrade() -> anyhow::Result<()> {
    tracing::info!("Upgrading FerrisHost...");
    // Upgrade logic to be implemented
    Ok(())
}

async fn uninstall(also_k3s: bool) -> anyhow::Result<()> {
    tracing::info!("Uninstalling FerrisHost...");
    if also_k3s {
        tracing::info!("Also uninstalling k3s");
    }
    // Uninstall logic to be implemented
    Ok(())
}
