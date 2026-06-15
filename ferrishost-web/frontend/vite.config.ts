import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  build: {
    outDir: "dist",
    target: "esnext",
  },
  server: {
    proxy: {
      "/api": "http://localhost:8080",
    },
  },
});
