use clap::{Parser, Subcommand};
use ferrishost_core::{ClusterStatus, GpuStatus, HostInfo};
use std::path::PathBuf;
use tracing_subscriber;

mod deploy_web;
mod gpu;
mod k3s;
mod operators;
mod system;

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

    // Verify root
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("This command must be run as root");
        std::process::exit(1);
    }

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
    web_port: u16,
    no_browser: bool,
    offline: bool,
) -> anyhow::Result<()> {
    tracing::info!("Starting FerrisHost setup...");

    // Step 1: Preflight checks
    tracing::info!("Running preflight checks...");
    let sys = SystemInfo::detect()?;
    sys.validate()?;

    // Step 2: GPU detection and host prep
    if !skip_gpu {
        tracing::info!("Detecting GPU configuration...");
        // GPU detection will be implemented in gpu module
    }

    // Step 3: Install k3s
    tracing::info!("Installing k3s...");
    // k3s installation will be implemented in k3s module

    // Step 4: Install core operators
    tracing::info!("Installing core operators...");
    // Operator installation will be implemented in operators module

    // Step 5: Deploy ferrishost-web
    tracing::info!("Deploying ferrishost-web...");
    // Web deployment will be implemented in deploy_web module

    // Step 6: Hand off
    tracing::info!("Setup complete!");

    Ok(())
}

async fn status() -> anyhow::Result<()> {
    println!("FerrisHost Status");
    println!("================\n");

    // This will be expanded to show actual status
    println!("Host info: [To be implemented]");
    println!("Kubernetes: [To be implemented]");
    println!("GPU status: [To be implemented]");
    println!("ferrishost-web: [To be implemented]");

    Ok(())
}

async fn gpu_command(detect_only: bool) -> anyhow::Result<()> {
    println!("GPU Detection");
    println!("=============\n");

    // This will detect GPU info
    println!("[GPU detection to be implemented]");

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
