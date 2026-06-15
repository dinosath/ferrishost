import type {
  ClusterStatus,
  GpuStatus,
  ModuleDescriptor,
  SetupState,
  JobStatusResponse,
  InstallResponse,
  HostInfo,
} from "./types";

const BASE = "/api";

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${url}`, {
    headers: { accept: "application/json" },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`${res.status} ${res.statusText}: ${text}`);
  }
  return res.json();
}

// ------------------------------------------------------------------ Status
export function fetchClusterStatus(): Promise<ClusterStatus> {
  return fetchJson("/status");
}

export function fetchGpuStatus(): Promise<GpuStatus> {
  return fetchJson("/gpu");
}

// ------------------------------------------------------------------ Setup
export function fetchSetupState(): Promise<SetupState> {
  return fetchJson("/setup");
}

export function updateSetupState(state: SetupState): Promise<SetupState> {
  return fetchJson("/setup", {
    method: "POST",
    body: JSON.stringify(state),
    headers: { "content-type": "application/json" },
  });
}

// ---------------------------------------------------------------- Modules
export function fetchModules(): Promise<ModuleDescriptor[]> {
  return fetchJson("/modules");
}

export function fetchModule(id: string): Promise<ModuleDescriptor> {
  return fetchJson(`/modules/${id}`);
}

export function installModule(id: string): Promise<InstallResponse> {
  return fetchJson(`/modules/${id}/install`, { method: "POST" });
}

export function uninstallModule(id: string): Promise<InstallResponse> {
  return fetchJson(`/modules/${id}/uninstall`, { method: "POST" });
}

// ------------------------------------------------------------------ Jobs
export function fetchJobStatus(name: string): Promise<JobStatusResponse> {
  return fetchJson(`/jobs/${name}`);
}
