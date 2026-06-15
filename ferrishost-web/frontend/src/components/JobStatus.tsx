import { createResource, createEffect, Component } from "solid-js";
import { fetchJobStatus } from "../lib/api";

interface Props {
  jobName: string;
  onComplete: () => void;
}

export const JobStatus: Component<Props> = (props) => {
  const [status, { refetch }] = createResource(
    () => props.jobName,
    fetchJobStatus,
    { initialValue: { name: props.jobName, active: 1, succeeded: 0, failed: 0, phase: "Running" as const } }
  );

  // Poll while running
  createEffect(() => {
    if (status()?.phase === "Running") {
      const timer = setInterval(refetch, 2000);
      return () => clearInterval(timer);
    }
  });

  createEffect(() => {
    if (status()?.phase === "Completed") {
      setTimeout(props.onComplete, 2000);
    }
  });

  return (
    <div class="flex items-center gap-2 text-sm">
      <span
        class={`inline-block w-2 h-2 rounded-full ${
          status()?.phase === "Completed"
            ? "bg-green-500"
            : status()?.phase === "Failed"
              ? "bg-red-500"
              : "bg-yellow-500 animate-pulse"
        }`}
      />
      <span>
        {status()?.phase === "Running" && "Installing..."}
        {status()?.phase === "Completed" && "Installed!"}
        {status()?.phase === "Failed" && "Failed"}
      </span>
    </div>
  );
};
