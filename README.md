# FerrisHost

FerrisHost is a lightweight, open-source personal cloud bootstrapper for self-hosted infrastructure. Go from a bare Linux box to a running personal cloud with one command.

## Features

- **100% Open Source** — No proprietary components, no vendor-operated backends
- **Lightweight by Default** — Minimal core (k3s + operators), everything else is opt-in
- **Single Rust Binary** — Fast, secure installer with minimal dependencies
- **GPU Support** — First-class support for NVIDIA and AMD GPUs
- **Easy Setup Wizard** — Web-based configuration after bootstrap

## Quick Start

### Shell Installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/dinosath/ferrishost/releases/latest/download/ferrishost-installer.sh \
  | sh
sudo ferrishost setup
```

> The installer places `ferrishost` in `~/.cargo/bin/`. If that directory
> is not yet on your `PATH`, use `sudo ~/.cargo/bin/ferrishost setup`.

### From Source

```bash
git clone https://github.com/dinosath/ferrishost
cd ferrishost
cargo build --release -p ferrishost-cli
sudo ./target/release/ferrishost setup
```

## Project Structure

This is a Cargo workspace with three main crates:

```
ferrishost/
├── ferrishost-cli/          # Bootstrap binary
├── ferrishost-web/          # In-cluster setup UI & dashboard (axum)
└── ferrishost-core/         # Shared types and constants
```

## Architecture

### Phase 1: ferrishost-cli (runs on host as root)

1. **Preflight** — Verify root, Linux, supported arch
2. **GPU Detection** — Detect NVIDIA/AMD GPUs and prepare host
3. **k3s Installation** — Install Kubernetes via k3s
4. **Core Operators** — Install cert-manager, GPU plugins, metrics-server
5. **Deploy Web UI** — Deploy ferrishost-web into the cluster
6. **Handoff** — Print URL and open browser; CLI's job is done

### Phase 2: ferrishost-web (runs in cluster)

- Serves setup wizard and dashboard
- Manages optional modules via Kubernetes API
- Handles TLS, admin account, and persistent configuration
- Stateless; uses ConfigMaps/Secrets for state

## Supported Components (v1)

### Core (Always Installed)

- k3s (Kubernetes)
- cert-manager (TLS)
- NVIDIA Device Plugin (if detected)
- AMD Device Plugin (if detected)
- metrics-server (built into k3s)
- Traefik Ingress Controller (built into k3s)
- local-path-provisioner (built into k3s)

### Optional Modules (Install via Web UI)

- Headscale (VPN/remote access)
- File sync apps (Seafile, Syncthing)
- Auth providers (SSO/OIDC)
- More coming soon...

## Development

### Build

```bash
# Build all crates
cargo build

# Build release
cargo build --release

# Build just the CLI
cargo build -p ferrishost-cli --release

# Build just the web service
cargo build -p ferrishost-web --release
```

### Test

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p ferrishost-cli
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Combined check
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

## Commands

### ferrishost setup [OPTIONS]

Bootstrap FerrisHost on a Linux system.

```
Options:
  --skip-gpu              Skip GPU detection/configuration
  --gpu-vendor <vendor>   Force GPU vendor (nvidia, amd, auto)
  --disable-traefik       Disable Traefik ingress controller
  --web-port <PORT>       Port for ferrishost-web (default: 443)
  --no-browser            Don't auto-open browser
  --offline               Skip network-dependent steps
```

### ferrishost status

Display host, k3s, GPU, and ferrishost-web status.

### ferrishost gpu [--detect-only]

Detect and report GPU information.

### ferrishost upgrade

Reconcile to current component versions.

### ferrishost uninstall [--also-k3s]

Uninstall FerrisHost (optionally removing k3s).

## Requirements

- **OS**: Linux (x86_64 or aarch64)
- **Root access** for installation
- **No other Kubernetes distribution** must be running
- **Minimum resources**: 2 CPUs, 2 GB RAM (more recommended)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [Olares](https://github.com/beclab/olares) — Inspiration and reference
- [k3s](https://k3s.io/) — Lightweight Kubernetes
