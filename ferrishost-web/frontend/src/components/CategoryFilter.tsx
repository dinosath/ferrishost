import { Component, For } from "solid-js";

const CATEGORIES = [
  "All",
  "networking",
  "storage",
  "vpn",
  "auth",
  "app",
  "monitoring",
];

export const CategoryFilter: Component = () => {
  return (
    <div>
      <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">
        Categories
      </h3>
      <ul class="space-y-1">
        <For each={CATEGORIES}>
          {(cat) => (
            <li>
              <button class="w-full text-left px-3 py-1.5 text-sm rounded hover:bg-gray-100 transition-colors">
                {cat}
              </button>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
};
