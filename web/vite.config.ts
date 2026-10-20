import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Two builds of the same screens:
// - "app": what the Tauri window loads (web/dist); Tauri itself adds the page rules of tauri.conf.json.
// - "screens": for screen tests in a plain browser (web/dist-screens), with the same page rules as a meta tag,
//   minus the app's channel to Rust, which a plain browser does not have.
const SCREENS_CSP =
  "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; " +
  "connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'";

export default defineConfig(({ command, mode }) => {
  // A build names its mode; other tools (the type checker) load this file without one.
  if (command === "build" && mode !== "app" && mode !== "screens") throw new Error(`unknown build mode: ${mode}`);
  return {
    plugins: [
      svelte(),
      {
        name: "screens-csp",
        transformIndexHtml: () =>
          mode === "screens"
            ? [{ tag: "meta", attrs: { "http-equiv": "Content-Security-Policy", content: SCREENS_CSP }, injectTo: "head-prepend" }]
            : [],
      },
    ],
    build: {
      outDir: mode === "app" ? "dist" : "dist-screens",
      emptyOutDir: true,
      modulePreload: { polyfill: false },
      assetsInlineLimit: 0,
    },
  };
});
