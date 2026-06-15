import { Component } from "solid-js";

export const SearchBar: Component = () => {
  return (
    <div>
      <input
        type="search"
        placeholder="Search modules..."
        class="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>
  );
};
