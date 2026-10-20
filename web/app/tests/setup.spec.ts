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

test("the_key_box_never_shows_a_saved_key", async ({ page }) => {
  await setupOf(page, "setup-key-saved");
  const box = page.getByLabel("Anthropic API key");
  await expect(page.getByText("Saved", { exact: true })).toBeVisible();
  await expect(box).toHaveValue("");
  await expect(box).toHaveAttribute("type", "password");
  const typed = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
  await box.fill(typed);
  await page.getByRole("button", { name: "Save key" }).click();
  await expect(box).toHaveValue("");
  expect(await page.content()).not.toContain("STEPLET-TEST");
});
