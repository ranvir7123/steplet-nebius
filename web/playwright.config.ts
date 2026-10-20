import { defineConfig, devices } from "@playwright/test";

// Screen tests run against the "screens" build in Chromium, the engine inside WebView2, at the normal size.
// Tests named real-*.spec.ts drive the real app window instead, through the debugging port of its e2e build
// (smaller choice S13); that build must be running before they start.
export default defineConfig({
  testDir: "app/tests",
  outputDir: "test-results",
  reporter: [["list"]],
  use: { baseURL: "http://127.0.0.1:4180" },
  webServer: {
    command: "npx --offline vite preview --mode screens --host 127.0.0.1 --port 4180 --strictPort",
    url: "http://127.0.0.1:4180",
    reuseExistingServer: false,
  },
  projects: [
    {
      name: "screens",
      testIgnore: /real-.*\.spec\.ts$/,
      use: { ...devices["Desktop Chrome"], viewport: { width: 1100, height: 800 } },
    },
    { name: "real", testMatch: /real-.*\.spec\.ts$/ },
  ],
});
