import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  server: { port: 1420, strictPort: true },
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_"],
  build: { target: "esnext" },
});