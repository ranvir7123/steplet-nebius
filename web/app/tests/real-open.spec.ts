import { chromium, expect, test } from "@playwright/test";
import { fileURLToPath } from "node:url";

// Drives the real app window through WebView2's debugging port. The e2e build of steplet.exe must be running
// (cargo build -p steplet-app --offline --features e2e, then target\debug\steplet.exe).
test("the real window opens on the empty Open screen", async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  try {
    const pages = browser.contexts().flatMap((c) => c.pages());
    expect(pages).toHaveLength(1);
    const page = pages[0];
    expect(page.url()).toBe("http://tauri.localhost/");
    expect(await page.title()).toBe("Steplet");
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Open a .steplet file to see what it asks for.");
    const dark = await page.evaluate(() => matchMedia("(prefers-color-scheme: dark)").matches);
    const name = `real-open-empty-${dark ? "dark" : "light"}.png`;
    await page.screenshot({ path: fileURLToPath(new URL(`../../../docs/screens/phase-4/${name}`, import.meta.url)) });
    console.log(`saved docs/screens/phase-4/${name}`);
  } finally {
    // For a browser reached over CDP this only disconnects; the app keeps running.
    await browser.close();
  }
});

test("the real window opens a file and shows what it asks for", async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  try {
    const page = browser.contexts()[0].pages()[0];
    const file = fileURLToPath(new URL("../../../examples/run-tests.steplet", import.meta.url));
    await page.locator("input[type=file]").setInputFiles(file);
    await expect(page.getByRole("heading", { name: "What Steplet found" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "From the file" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Run", exact: true })).toBeVisible();
    expect(await page.title()).toBe("Steplet");
    const dark = await page.evaluate(() => matchMedia("(prefers-color-scheme: dark)").matches);
    const name = `real-preview-${dark ? "dark" : "light"}.png`;
    await page.screenshot({
      path: fileURLToPath(new URL(`../../../docs/screens/phase-4/${name}`, import.meta.url)),
      fullPage: true,
    });
    console.log(`saved docs/screens/phase-4/${name}`);
    // Back to the empty Open screen, where the first test expects the window.
    await page.getByRole("button", { name: "Open another file" }).click();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Open a .steplet file to see what it asks for.");
  } finally {
    await browser.close();
  }
});

test("the real window reaches nothing outside the app", async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  try {
    const context = browser.contexts()[0];
    const page = context.pages()[0];
    // A request the web view starts is not enough to fail: what must never happen is an answer from outside.
    // Each try uses an address never asked for before, so no copy in the web view's cache can answer it.
    const fresh = (what: string) => `https://example.com/?steplet-test=${what}-${Date.now()}`;
    const answered: string[] = [];
    page.on("response", (r) => answered.push(r.url()));
    const warnings: string[] = [];
    page.on("console", (m) => warnings.push(m.text()));

    // The page rules refuse a request to the internet before it is made.
    const fetched = await page.evaluate((u) =>
      fetch(u).then(
        () => "reached",
        (e) => `refused: ${e}`,
      ),
      fresh("fetch"),
    );
    expect(fetched).toMatch(/^refused/);

    // A new window is refused.
    const opened = await page.evaluate((u) => window.open(u) === null, fresh("open"));
    expect(opened).toBe(true);

    // Leaving the app's own page is refused, and the try reaches nothing (measured 21 Oct 2026: WebView2 sends
    // the request before the app's navigation check cancels it, so the check alone is not enough).
    await page.evaluate((u) => {
      location.href = u;
    }, fresh("navigate"));
    await page.waitForTimeout(3000);
    expect(page.url()).toBe("http://tauri.localhost/");
    expect(context.pages()).toHaveLength(1);

    expect(answered.filter((u) => !u.startsWith("http://tauri.localhost/"))).toEqual([]);

    // The app's own channel to Rust still works through the closed-port proxy: a call reaches Tauri on its own
    // channel (http://ipc.localhost, which the web view hands to Tauri without the network), and Tauri refuses the
    // unknown command. Tauri would quietly fall back to another channel if that one failed, saying so on the console,
    // so the test also requires the channel's own answer and no such warning.
    const ipc = await page.evaluate(() =>
      (window as unknown as { __TAURI_INTERNALS__: { invoke(c: string): Promise<unknown> } }).__TAURI_INTERNALS__
        .invoke("no_such_command")
        .then(
          () => "answered",
          (e) => `refused by Tauri: ${e}`,
        ),
    );
    expect(ipc).toMatch(/^refused by Tauri: .*no_such_command/);
    expect(answered).toContain("http://ipc.localhost/no_such_command");
    expect(warnings.filter((w) => w.includes("IPC custom protocol failed"))).toEqual([]);
  } finally {
    await browser.close();
  }
});
