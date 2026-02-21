import { defineConfig } from "vite";
// Touching file to force reload
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from '@tailwindcss/vite';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [sveltekit(), tailwindcss()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 3000,
    strictPort: true,
    host: "127.0.0.1",
    hmr: {
      host: "127.0.0.1",
      port: 3001,
    },
    watch: {
      // 3. tell Vite to ignore watching `src-tauri` and noisy logs/temp files
      ignored: [
        "**/src-tauri/**",
        "**/*.log",
        "**/*.csv",
        "**/temp_*",
        "**/rtd_debug.log",
        "**/.svelte-kit/**",
        "**/node_modules/.vite/**"
      ],
    },
  },
}));
