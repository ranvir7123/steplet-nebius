import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The public check page's lock: it may load its own files and run its embedded WASM, and nothing may leave.
const CHECK_PAGE_CSP =
  "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; " +
  "connect-src 'none'; form-action 'none'; base-uri 'none'";

export default defineConfig(({ mode }) => ({
  plugins: [
    svelte(),
    {
      name: "check-page-csp",
      transformIndexHtml: () =>
        mode === "browser"
          ? [{ tag: "meta", attrs: { "http-equiv": "Content-Security-Policy", content: CHECK_PAGE_CSP }, injectTo: "head-prepend" }]
          : [],
    },
  ],
  build: {
    outDir: mode === "browser" ? "dist" : "dist-tauri",
    modulePreload: { polyfill: false },
    assetsInlineLimit: 0,
  },
}));
