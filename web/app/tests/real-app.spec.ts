import { chromium, expect, test, type Browser, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { basename, dirname } from "node:path";
import { fileURLToPath } from "node:url";

// Task 15: use case 1 in the real window, driven through the debugging port of the e2e build (S13).
// "setup" runs in a window started without a file. "use case 1" runs in the window Windows opened for
// examples\sample-run.steplet (Invoke-Item, the path a double-click takes), started with STEPLET_SAVE_TO set to the
// folder the report is saved in. Only one e2e window may be open at a time: both use port 9333.
// The key is never touched here: the owner pastes it into the key box themself.

async function attach(): Promise<{ browser: Browser; page: Page }> {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  const pages = browser.contexts().flatMap((c) => c.pages());
  expect(pages).toHaveLength(1);
  expect(pages[0].url()).toBe("http://tauri.localhost/");
  return { browser, page: pages[0] };
}

/** A picture of the real window, `real-uc1-<screen>-<light|dark>.png`; the mouse leaves the page first. */
async function picture(page: Page, screen: string, fullPage = true) {
  await page.mouse.move(0, 0);
  const dark = await page.evaluate(() => matchMedia("(prefers-color-scheme: dark)").matches);
  const name = `real-uc1-${screen}-${dark ? "dark" : "light"}.png`;
  await page.screenshot({ path: fileURLToPath(new URL(`../../../docs/screens/phase-4/${name}`, import.meta.url)), fullPage });
  console.log(`saved docs/screens/phase-4/${name}`);
}

test("setup: Steplet's Linux has this build's gate, and .steplet files open with this build", async () => {
  test.setTimeout(20 * 60_000);
  const { browser, page } = await attach();
  try {
    const setup = page.getByRole("heading", { name: "Set up Steplet" });
    if (!(await setup.isVisible())) await page.getByRole("button", { name: "Setup" }).click({ timeout: 10_000 });
    await expect(setup).toBeVisible();
    await expect(page.getByText(/^Windows' Linux feature \(WSL \d+\.\d+\.\d+/)).toBeVisible({ timeout: 90_000 });
    const lines = page.locator(".checklist li");
    for (const l of await lines.allInnerTexts()) console.log(`setup line: ${l.replace(/\n/g, " | ")}`);

    const finish = page.getByRole("button", { name: "Finish setting up Steplet's Linux" });
    if (await finish.isVisible()) {
      await finish.click();
      await expect(page.getByText("Setup finished.")).toBeVisible({ timeout: 18 * 60_000 });
      const printed = await page.getByRole("log", { name: "What setup printed" }).innerText();
      for (const l of printed.split("\n")) console.log(`setup printed: ${l}`);
    }
    // After an install the screen looks at this computer again; every line must then be ready.
    await expect
      .poll(async () => (await lines.allInnerTexts()).filter((l) => !l.startsWith("Ready")), { timeout: 90_000 })
      .toEqual([]);
    for (const l of await lines.allInnerTexts()) console.log(`setup line now: ${l.replace(/\n/g, " | ")}`);

    const point = page.getByRole("button", { name: "Point it at this Steplet" });
    const opens = page.getByRole("checkbox", { name: "Open .steplet files with Steplet" });
    if (await point.isVisible()) await point.click();
    else if (!(await opens.isChecked())) await opens.check();
    await expect(point).toHaveCount(0);
    await expect(opens).toBeChecked();

    const keySaved = await page.getByText("Saved", { exact: true }).isVisible();
    console.log(`key saved: ${keySaved ? "yes" : "no"}`);
    await picture(page, "setup");
  } finally {
    // For a browser reached over CDP this only disconnects; the app keeps running.
    await browser.close();
  }
});

test("use case 1: preview, chooser, prepare, the run and the report, saved", async () => {
  test.setTimeout(45 * 60_000);
  const folder = process.env.STEPLET_SAVE_TO;
  expect(folder, "STEPLET_SAVE_TO names the folder the app was started with").toBeTruthy();
  const { browser, page } = await attach();
  try {
    // Windows started Steplet with the file: its preview is the first screen.
    await expect(page.getByRole("heading", { name: "What Steplet found" })).toBeVisible({ timeout: 30_000 });
    await expect(page.locator(".title-text")).toHaveText("Run the tests of steplet-sample");
    expect(await page.title()).toBe("Steplet");
    await picture(page, "preview");

    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Choose how it runs" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Run with Claude Code" })).toBeEnabled({ timeout: 90_000 });
    await picture(page, "chooser");
    await page.getByRole("button", { name: "Run with Claude Code" }).click();

    await expect(page.getByRole("heading", { name: "Getting the repository ready" })).toBeVisible();
    await expect(page.getByText(/^Fetched/).first()).toBeVisible({ timeout: 5 * 60_000 });
    await picture(page, "prepare", false);

    // The driven run approves nothing the file did not ask for: every question gets No, and is recorded.
    const end = page.locator("#end-title");
    const failed = page.locator("#failed-title");
    const log = page.getByRole("log", { name: "What happens in the run" });
    let asked = 0;
    let pictured = false;
    while (!(await end.isVisible())) {
      if (!pictured && (await log.locator("li").count()) >= 2) {
        pictured = true;
        await picture(page, "run-live", false);
      }
      if (await failed.isVisible()) throw new Error(`the run could not start: ${await failed.locator("..").innerText()}`);
      const question = page.locator("dialog.ask[open]");
      if (await question.isVisible()) {
        asked += 1;
        console.log(`question ${asked}: ${(await question.innerText()).replace(/\n+/g, " | ")}`);
        if (asked === 1) await picture(page, "ask", false);
        await question.getByRole("button", { name: "No", exact: true }).click();
        await expect(question).toBeHidden({ timeout: 10_000 });
      }
      await page.waitForTimeout(1000);
    }
    console.log(`questions answered No: ${asked}`);
    console.log(`end: ${(await end.locator("..").innerText()).replace(/\n+/g, " | ")}`);
    await picture(page, "run-ended", false);

    await page.getByRole("button", { name: "See the report" }).click();
    await expect(page.getByRole("article", { name: "The report" })).toBeVisible();
    await picture(page, "report");

    await page.getByRole("button", { name: "Save report…" }).click();
    const saved = page.getByRole("status").filter({ hasText: /^Saved as / });
    await expect(saved).toBeVisible({ timeout: 30_000 });
    const path = (await saved.innerText()).replace(/^Saved as /, "").trim();
    console.log(`saved report: ${path}`);
    expect(dirname(path)).toBe(folder);
    expect(basename(path)).toMatch(/^steplet-report-\d{4}-\d{2}-\d{2}-\d{4}\.txt$/);
    await picture(page, "report-saved", false);

    // The test command's real exit code, as the saved report gives it.
    const lines = readFileSync(path, "utf8").split("\r\n");
    const asks = lines.flatMap((l, i) => (l === "     | python3 -m unittest discover -s tests -t ." ? [i] : []));
    expect(asks.length, "the saved report holds the file's test command").toBeGreaterThan(0);
    for (const i of asks) for (const l of lines.slice(i - 4, i + 3)) console.log(`report: ${l}`);
    expect(lines[asks[asks.length - 1] + 2]).toBe("     Exit code 0");
  } finally {
    await browser.close();
  }
});
