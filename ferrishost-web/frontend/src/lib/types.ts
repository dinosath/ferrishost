/// Core types matching ferrishost-core structs

export interface HostInfo {
  os: string;
  kernel_version: string;
  arch: string;
  hostname: string;
}

export interface GpuInfo {
  vendor: string;
  name: string;
  memory_mb: number;
  index: number;
}

export interface GpuStatus {
  detected: GpuInfo[];
  nvidia_available: boolean;
  amd_available: boolean;
}

export interface NodeStatus {
  name: string;
  ready: boolean;
  cpu_millis: number;
  memory_mb: number;
}

export interface ClusterStatus {
  k3s_installed: boolean;
  k3s_version: string | null;
  nodes: NodeStatus[];
  all_nodes_ready: boolean;
}

export interface ModuleDescriptor {
  id: string;
  name: string;
  description: string;
  category: string;
  version: string;
  installed: boolean;
  icon?: string;
}

export interface SetupState {
  domain: string | null;
  tls_mode: "self-signed" | "lets-encrypt";
  admin_username: string | null;
  admin_password_hash: string | null;
  timezone: string | null;
  completed: boolean;
}

export interface JobStatusResponse {
  name: string;
  active: number;
  succeeded: number;
  failed: number;
  phase: "Running" | "Completed" | "Failed";
}

export interface InstallResponse {
  status: string;
  module: string;
  jobName?: string;
}
