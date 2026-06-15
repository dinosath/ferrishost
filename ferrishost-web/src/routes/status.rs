use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use ferrishost_core::{ClusterStatus, GpuStatus};

use crate::state::AppState;

/// GET /api/status — return cluster status.
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Json<ClusterStatus> {
    let k3s_installed = state.k8s.is_some();

    Json(ClusterStatus {
        k3s_installed,
        k3s_version: None,
        nodes: vec![],
        all_nodes_ready: false,
    })
}

/// GET /api/gpu — return GPU status.
pub async fn get_gpu_status() -> Json<GpuStatus> {
    Json(GpuStatus {
        detected: vec![],
        nvidia_available: false,
        amd_available: false,
    })
}
