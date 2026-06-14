use axum::Json;
use ferrishost_core::{GpuStatus, HostInfo, ClusterStatus, GpuInfo, NodeStatus};
use std::sync::Arc;
use crate::state::AppState;

pub async fn get_status() -> Json<ClusterStatus> {
    Json(ClusterStatus {
        k3s_installed: false,
        k3s_version: None,
        nodes: vec![],
        all_nodes_ready: false,
    })
}

pub async fn get_gpu_status() -> Json<GpuStatus> {
    Json(GpuStatus {
        detected: vec![],
        nvidia_available: false,
        amd_available: false,
    })
}
