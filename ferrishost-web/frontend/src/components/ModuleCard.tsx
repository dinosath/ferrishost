import { Component } from "solid-js";
import { A } from "@solidjs/router";
import { InstallButton } from "./InstallButton";
import type { ModuleDescriptor } from "../lib/types";

export const ModuleCard: Component<{ module: ModuleDescriptor }> = (props) => {
  return (
    <div class="rounded-lg border p-4 hover:shadow-md transition-shadow bg-white">
      <div class="flex items-center gap-3 mb-3">
        <div class="w-12 h-12 bg-gray-100 rounded flex items-center justify-center text-xl">
          📦
        </div>
        <div>
          <A href={`/store/${props.module.id}`} class="font-semibold hover:text-blue-600 no-underline text-inherit">
            {props.module.name}
          </A>
          <span class="text-sm text-gray-500 block">{props.module.category}</span>
        </div>
      </div>
      <p class="text-sm mb-4 text-gray-700">{props.module.description}</p>
      <div class="flex justify-between items-center">
        <span class="text-xs text-gray-400">v{props.module.version}</span>
        <InstallButton module={props.module} />
      </div>
    </div>
  );
};
