import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Fixture } from "../../shared/types/Fixture";

// The Setup screen (Task 11), reached from the "Setup" link at the top of every screen. Against the screens build and
// its stand-in backend, which replays data saved from real Rust output (web/app/tests/fixtures).

const saved = (name: string): Fixture =>
  JSON.parse(readFileSync(new URL(`./fixtures/${name}.json`, import.meta.url), "utf8")) as Fixture;

async function setupOf(page: Page, scenario: string) {
  await page.goto(`/?scenario=${scenario}`);
  await page.getByRole("button", { name: "Setup" }).click();
  await expect(page.getByRole("heading", { name: "Set up Steplet" })).toBeVisible();
}

test("wsl_missing_shows_the_command_to_type", async ({ page }) => {
  await setupOf(page, "setup-wsl-missing");
  await expect(page.locator("code", { hasText: "wsl --install --no-distribution" })).toBeVisible();
  await expect(page.getByText("opened as administrator")).toBeVisible();
  // Steplet never runs it, and offers no install of its own Linux before Windows' feature is there.
  await expect(page.getByRole("button", { name: /Install/ })).toHaveCount(0);
});

test("install_progress_then_failure_offers_try_again_and_start_over", async ({ page }) => {
  await setupOf(page, "setup-install-failed");
  // Before Install: what it adds to Windows, said first (Task 6's ruling).
  await expect(page.getByText(/Windows Terminal/)).toBeVisible();
  await page.getByRole("button", { name: "Install Steplet's Linux" }).click();
  const log = page.getByRole("log");
  const f = saved("setup-install-failed");
  await expect(log.locator("[data-from-file]")).toHaveCount(f.setup_lines.length);
  await expect(page.getByRole("alert")).toContainText(f.setup_error!.reason);
  await expect(page.getByRole("alert").locator("[data-from-file]")).toHaveCount(1);
  await expect(page.getByRole("button", { name: "Try again" })).toBeVisible();
  await page.getByRole("button", { name: "Start over" }).click();
  // Start over removes Steplet's own Linux, so it asks first.
  await expect(page.getByRole("button", { name: "Remove Steplet's Linux" })).toBeVisible();
  await expect(page.getByText(/Nothing of yours is in it/)).toBeVisible();
});

const nebiusBox = (page: Page) => page.getByRole("region", { name: "Nebius Token Factory key" });
const anthropicBox = (page: Page) => page.getByRole("region", { name: "Anthropic key, for Claude Code" });
const SENT_TO = "Steplet sends it only to api.tokenfactory.nebius.com.";

test("the_key_box_never_shows_a_saved_key", async ({ page }) => {
  await setupOf(page, "setup-key-saved");
  const panel = anthropicBox(page);
  const box = panel.getByLabel("Anthropic API key");
  await expect(panel.getByText("Saved", { exact: true })).toBeVisible();
  await expect(box).toHaveValue("");
  await expect(box).toHaveAttribute("type", "password");
  const typed = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
  await box.fill(typed);
  await panel.getByRole("button", { name: "Save key" }).click();
  await expect(box).toHaveValue("");
  expect(await page.content()).not.toContain("STEPLET-TEST");
});

test("the_nebius_key_has_its_own_box_first", async ({ page }) => {
  await setupOf(page, "setup-nothing-done");
  await expect(page.getByRole("heading", { level: 2 })).toHaveText([
    "Steplet's own Linux",
    "Nebius Token Factory key",
    "Anthropic key, for Claude Code",
    "Opening .steplet files",
  ]);
  const panel = nebiusBox(page);
  const box = panel.getByLabel("Nebius API key");
  await expect(box).toHaveValue("");
  await expect(box).toHaveAttribute("type", "password");
  await expect(panel.getByRole("button", { name: "Save key" })).toBeDisabled();
  await expect(panel.getByRole("button", { name: "Forget key" })).toHaveCount(0);
  await box.fill("  nebius-STEPLET-TEST-MADE-UP-NOT-A-KEY  ");
  await panel.getByRole("button", { name: "Save key" }).click();
  await expect(panel.getByText(SENT_TO)).toBeVisible();
  await expect(box).toHaveValue("");
  await expect(box).toHaveAttribute("placeholder", "Paste a new key to replace it");
  expect(await page.content()).not.toContain("STEPLET-TEST");
  // Only the Nebius key was saved.
  await expect(anthropicBox(page).getByText(/Saved/)).toHaveCount(0);
  await panel.getByRole("button", { name: "Forget key" }).click();
  await expect(panel.getByText(/Saved/)).toHaveCount(0);
  await expect(panel.getByRole("button", { name: "Forget key" })).toHaveCount(0);
});

test("saving_or_forgetting_one_key_leaves_the_other_alone", async ({ page }) => {
  await setupOf(page, "setup-nebius-key-saved");
  await expect(nebiusBox(page).getByText(SENT_TO)).toBeVisible();
  const anthropic = anthropicBox(page);
  await expect(anthropic.getByText(/Saved/)).toHaveCount(0);
  await anthropic.getByLabel("Anthropic API key").fill("sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY");
  await anthropic.getByRole("button", { name: "Save key" }).click();
  await expect(anthropic.getByText("Saved", { exact: true })).toBeVisible();
  await expect(nebiusBox(page).getByText(SENT_TO)).toBeVisible();
  await anthropic.getByRole("button", { name: "Forget key" }).click();
  await expect(anthropic.getByText(/Saved/)).toHaveCount(0);
  await expect(nebiusBox(page).getByText(SENT_TO)).toBeVisible();
});

test("a_refused_key_says_why_in_its_own_box", async ({ page }) => {
  await setupOf(page, "setup-key-refused");
  const panel = nebiusBox(page);
  const box = panel.getByLabel("Nebius API key");
  await box.fill("two words");
  await panel.getByRole("button", { name: "Save key" }).click();
  await expect(panel.getByRole("alert")).toHaveText(saved("setup-key-refused").key_error!);
  await expect(box).toHaveValue("");
  await expect(panel.getByText(/Saved/)).toHaveCount(0);
  await expect(anthropicBox(page).getByRole("alert")).toHaveCount(0);
  expect(await page.content()).not.toContain("two words");
});

test("a_switch_that_could_not_change_shows_how_it_stands", async ({ page }) => {
  // Windows refused the change: the box must not show it as done.
  await setupOf(page, "setup-switch-fails");
  const box = page.getByLabel("Open .steplet files with Steplet");
  await expect(box).not.toBeChecked();
  await box.click();
  await expect(page.getByRole("alert")).toContainText(saved("setup-switch-fails").switch_error!);
  await expect(box).not.toBeChecked();
});

test("leaving_setup_during_an_install_keeps_it_going", async ({ page }) => {
  await setupOf(page, "setup-installing");
  await page.getByRole("button", { name: "Install Steplet's Linux" }).click();
  const lines = saved("setup-installing").setup_lines.length;
  await expect(page.getByRole("log").locator("[data-from-file]")).toHaveCount(lines);
  await page.getByRole("button", { name: "Back" }).click();
  await page.getByRole("button", { name: "Setup" }).click();
  // Coming back shows the same install, still going, with what it printed; nothing offers to start another.
  await expect(page.getByText("Steplet is changing its Linux. This can take several minutes…")).toBeVisible();
  await expect(page.getByRole("log").locator("[data-from-file]")).toHaveCount(lines);
  await expect(page.getByRole("button", { name: /Install Steplet/ })).toHaveCount(0);
});

test("wsl_too_old_says_to_update_it", async ({ page }) => {
  // Found by looking (Task 14): the help said the feature "has to be on", though it is on and only too old.
  await setupOf(page, "setup-wsl-too-old");
  await expect(page.locator("code", { hasText: "wsl --update" })).toBeVisible();
  await expect(page.getByText(/too old/)).toBeVisible();
  await expect(page.getByText("Then press Check again.", { exact: true })).toBeVisible();
  await expect(page.getByText(/has to be on/)).toHaveCount(0);
});
