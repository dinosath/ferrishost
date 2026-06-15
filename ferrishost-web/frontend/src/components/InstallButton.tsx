import { Component, createSignal, Show } from "solid-js";
import { installModule } from "../lib/api";
import { JobStatus } from "./JobStatus";
import type { ModuleDescriptor } from "../lib/types";

export const InstallButton: Component<{ module: ModuleDescriptor }> = (props) => {
  const [jobId, setJobId] = createSignal<string | null>(null);
  const [installing, setInstalling] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const handleInstall = async () => {
    setInstalling(true);
    setError(null);
    try {
      const result = await installModule(props.module.id);
      if (result.jobName) {
        setJobId(result.jobName);
      }
    } catch (e) {
      setError(String(e));
      setInstalling(false);
    }
  };

  return (
    <Show
      when={!jobId()}
      fallback={
        <JobStatus
          jobName={jobId()!}
          onComplete={() => {
            setJobId(null);
            setInstalling(false);
          }}
        />
      }
    >
      <Show when={!error()} fallback={<span class="text-xs text-red-500">{error()}</span>}>
        <button
          onClick={handleInstall}
          disabled={installing() || props.module.installed}
          class={`px-3 py-1.5 text-sm rounded font-medium transition-colors ${
            props.module.installed
              ? "bg-gray-100 text-gray-400 cursor-default"
              : "bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
          }`}
        >
          {installing() ? "Installing..." : props.module.installed ? "Installed" : "Install"}
        </button>
      </Show>
    </Show>
  );
};
