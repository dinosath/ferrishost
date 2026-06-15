import { createResource } from "solid-js";
import { useParams, A } from "@solidjs/router";
import { fetchModule } from "../lib/api";
import { InstallButton } from "../components/InstallButton";

export function ModuleDetail() {
  const params = useParams<{ id: string }>();
  const [mod] = createResource(() => params.id, fetchModule);

  return (
    <div class="max-w-3xl mx-auto p-6">
      <A href="/store" class="text-sm text-blue-600 hover:underline mb-4 inline-block">
        &larr; Back to store
      </A>

      <div class="bg-white rounded-lg border p-6">
        <div class="flex items-center gap-4 mb-4">
          <div class="w-16 h-16 bg-gray-100 rounded-lg flex items-center justify-center text-2xl">
            📦
          </div>
          <div>
            <h1 class="text-2xl font-bold">{mod()?.name}</h1>
            <span class="text-sm text-gray-500">{mod()?.category} &middot; v{mod()?.version}</span>
          </div>
        </div>

        <p class="text-gray-700 mb-6">{mod()?.description}</p>

        <div class="flex gap-3">
          <InstallButton module={mod()!} />
        </div>
      </div>
    </div>
  );
}
