import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import Icons from "unplugin-icons/vite";
import { resolve } from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  // Fluent UI System Icons (Microsoft, MIT) compiled to Svelte components — see src/lib/ui/icons.ts
  plugins: [svelte(), tailwindcss(), Icons({ compiler: "svelte" })],
  resolve: {
    alias: {
      $lib: resolve(__dirname, "src/lib"),
    },
  },
  clearScreen: false,
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1431 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
