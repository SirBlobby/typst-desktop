import { defineConfig } from "vite";
import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const svelteKitOptions = {
  preprocess: vitePreprocess(),
  adapter: adapter({
    fallback: "index.html",
  }),
};

export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit(svelteKitOptions), wasm()],

  optimizeDeps: {
    exclude: ["codemirror-lang-typst"],
    esbuildOptions: {
      target: "esnext",
    },
  },
  build: {
    target: "esnext",
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
