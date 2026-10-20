import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Fixture } from "../../shared/types/Fixture";

// The run chooser (Task 11), reached as a receiver reaches it: from the preview, by pressing Run. Against the screens
// build and its stand-in backend, which replays data saved from real Rust output (web/app/tests/fixtures).

const saved = (name: string): Fixture =>
  JSON.parse(readFileSync(new URL(`./fixtures/${name}.json`, import.meta.url), "utf8")) as Fixture;
const SENTENCE = "I understand the risks and want to run this file";

async function chooserOf(page: Page, scenario: string) {
  await page.goto(`/?scenario=${scenario}`);
  const sentence = page.getByLabel(/type this sentence exactly/);
  if (await sentence.count()) await sentence.fill(SENTENCE);
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Choose how it runs" })).toBeVisible();
}

test("codex_is_shown_unavailable_with_its_reason", async ({ page }) => {
  await chooserOf(page, "chooser-ready-no-sites");
  const codex = page.getByRole("region", { name: "Codex" });
  await expect(codex).toContainText(saved("chooser-ready-no-sites").chooser!.codex_reason);
  await expect(codex.getByRole("button")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Run with Claude Code" })).toBeEnabled();
});

test("only_the_lines_promises_returns_are_shown", async ({ page }) => {
  await chooserOf(page, "chooser-ready-15-sites");
  const promises = saved("chooser-ready-15-sites").chooser!.claude.promises;
  const lines = page.getByRole("list", { name: "What Steplet makes sure of in this run" }).getByRole("listitem");
  await expect(lines).toHaveCount(promises.length);
  for (const [i, p] of promises.entries()) {
    await expect(lines.nth(i)).toContainText(p.text);
    // The file's own sites, each as file text inside its line.
    const sites = lines.nth(i).locator("[data-from-file]");
    await expect(sites).toHaveCount(p.items.length);
    for (const [j, item] of p.items.entries()) {
      await expect(sites.nth(j)).toHaveText(item.map((s) => (s.kind === "text" ? s.text : "")).join(""));
    }
    // R6: each line says which tests back it.
    for (const t of p.tests) await expect(lines.nth(i)).toContainText(t);
  }
});

test("a_runner_that_is_not_set_up_sends_you_to_setup", async ({ page }) => {
  await chooserOf(page, "chooser-needs-setup");
  await expect(page.getByRole("button", { name: "Run with Claude Code" })).toHaveCount(0);
  await page.getByRole("button", { name: "Set up first" }).click();
  await expect(page.getByRole("heading", { name: "Set up Steplet" })).toBeVisible();
});
