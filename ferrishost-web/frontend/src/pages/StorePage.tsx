import { createResource, For, Show } from "solid-js";
import { fetchModules } from "../lib/api";
import { ModuleCard } from "../components/ModuleCard";
import { SearchBar } from "../components/SearchBar";
import { CategoryFilter } from "../components/CategoryFilter";

export function StorePage() {
  const [modules] = createResource(fetchModules);

  return (
    <div class="flex h-full">
      {/* Sidebar */}
      <aside class="w-64 border-r p-4">
        <CategoryFilter />
      </aside>

      {/* Main content */}
      <main class="flex-1 p-6">
        <SearchBar />

        <Show when={!modules.loading} fallback={
          <div class="flex justify-center mt-12">
            <span class="text-gray-500">Loading modules...</span>
          </div>
        }>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-6">
            <For each={modules()}>
              {(mod) => <ModuleCard module={mod} />}
            </For>
          </div>
        </Show>
      </main>
    </div>
  );
}
