import { chromium, expect, test } from "@playwright/test";

// The Nebius plan, Task 3: the Nebius key box in the real window, driven through the debugging port of the e2e build
// (S13), started without a file. It saves a made-up key through the real save_key command, sees the box say so, and
// forgets it again through forget_key. It never runs when a Nebius key is already saved, so the owner's key is never
// replaced, and it never touches the Anthropic box or the file-type switch.

const MADE_UP_KEY = "nebius-STEPLET-TEST-MADE-UP-NOT-A-KEY";
const SENT_TO = "Steplet sends it only to api.tokenfactory.nebius.com.";

test("the_nebius_key_box_saves_and_forgets_through_the_real_commands", async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  const pages = browser.contexts().flatMap((c) => c.pages());
  expect(pages).toHaveLength(1);
  const page = pages[0];
  expect(page.url()).toBe("http://tauri.localhost/");
  const nebius = page.getByRole("region", { name: "Nebius Token Factory key" });
  const anthropicSaved = page
    .getByRole("region", { name: "Anthropic key, for Claude Code" })
    .getByText("Saved", { exact: true });
  let saved = false;
  try {
    const setup = page.getByRole("heading", { name: "Set up Steplet" });
    if (!(await setup.isVisible())) await page.getByRole("button", { name: "Setup" }).click({ timeout: 10_000 });
    await expect(setup).toBeVisible();
    const box = nebius.getByLabel("Nebius API key");
    await expect(box).toBeVisible({ timeout: 90_000 });
    test.skip(await nebius.getByText(SENT_TO).isVisible(), "a Nebius key is already saved; this test never replaces it");
    const anthropicBefore = await anthropicSaved.isVisible();
    console.log(`Anthropic key saved before: ${anthropicBefore ? "yes" : "no"}`);

    await box.fill(MADE_UP_KEY);
    await nebius.getByRole("button", { name: "Save key" }).click();
    saved = true;
    await expect(nebius.getByText(SENT_TO)).toBeVisible();
    await expect(box).toHaveValue("");
    expect(await page.content()).not.toContain("STEPLET-TEST");
    expect(await anthropicSaved.isVisible()).toBe(anthropicBefore);
    console.log("made-up Nebius key: saved, and the box says so");

    await nebius.getByRole("button", { name: "Forget key" }).click();
    await expect(nebius.getByText(/Saved/)).toHaveCount(0);
    saved = false;
    expect(await anthropicSaved.isVisible()).toBe(anthropicBefore);
    console.log("made-up Nebius key: forgotten; the Anthropic box did not change");
  } finally {
    // A failure after saving must not leave the made-up key in the Nebius entry.
    const forget = nebius.getByRole("button", { name: "Forget key" });
    if (saved && (await forget.isVisible())) await forget.click();
    // For a browser reached over CDP this only disconnects; the app keeps running.
    await browser.close();
  }
});
