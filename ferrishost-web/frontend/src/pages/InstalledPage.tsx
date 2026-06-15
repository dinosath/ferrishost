import { createResource, For, Show } from "solid-js";
import { fetchModules } from "../lib/api";

export function InstalledPage() {
  const [modules] = createResource(fetchModules);
  const installed = () => modules()?.filter((m) => m.installed) ?? [];

  return (
    <div class="max-w-4xl mx-auto p-6">
      <h1 class="text-2xl font-bold mb-6">Installed Modules</h1>

      <Show when={modules.loading} fallback={
        <Show when={installed().length > 0} fallback={
          <div class="text-center py-12 text-gray-500">
            <p class="text-lg">No modules installed yet.</p>
            <p class="mt-2">Browse the store to find modules to install.</p>
          </div>
        }>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <For each={installed()}>
              {(mod) => (
                <div class="bg-white rounded-lg border p-4 flex items-center gap-4">
                  <div class="flex-1">
                    <h3 class="font-semibold">{mod.name}</h3>
                    <p class="text-sm text-gray-500">{mod.description}</p>
                  </div>
                  <span class="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">
                    v{mod.version}
                  </span>
                </div>
              )}
            </For>
          </div>
        </Show>
      }>
        <div class="flex justify-center py-12">
          <span class="text-gray-500">Loading...</span>
        </div>
      </Show>
    </div>
  );
}
