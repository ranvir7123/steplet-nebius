import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Fixture } from "../../shared/types/Fixture";
import type { ReportView } from "../../shared/types/ReportView";
import type { Seg } from "../../shared/types/Seg";

// The report (Task 13), reached as a receiver reaches it: the preview, Run, "Run with Claude Code", and the run's end.
// Against the screens build and its stand-in backend, which replays data saved from real Rust output.

const saved = (name: string): Fixture =>
  JSON.parse(readFileSync(new URL(`./fixtures/${name}.json`, import.meta.url), "utf8")) as Fixture;
const plain = (segs: Seg[]) => segs.map((s) => (s.kind === "text" ? s.text : "")).join("");

function reportOf(f: Fixture): ReportView {
  for (const p of f.progress) if (p.kind === "finished") return p.report;
  throw new Error("the scenario has no report");
}

async function reportScreen(page: Page, scenario: string) {
  await page.goto(`/?scenario=${scenario}`);
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await page.getByRole("button", { name: "Run with Claude Code" }).click();
  await page.getByRole("button", { name: "See the report" }).click();
  await expect(page.getByRole("article", { name: "The report" })).toBeVisible();
}

const sections = (page: Page) => page.getByRole("article", { name: "The report" }).getByRole("region");

test("the_agent_says_comes_last_and_is_labelled", async ({ page }) => {
  await reportScreen(page, "report-finished");
  const r = reportOf(saved("report-finished"));
  const last = sections(page).last();
  await expect(last.getByRole("heading")).toHaveText("The agent says");
  await expect(last).toContainText("Its own words, not Steplet's");
  await expect(last.locator("[data-from-file]")).toHaveText(plain(r.agent_says));
  // The outcome comes first, in Steplet's words.
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(r.end_words.title);
});

test("an_interrupted_run_offers_run_again", async ({ page }) => {
  await reportScreen(page, "report-interrupted");
  const r = reportOf(saved("report-interrupted"));
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Interrupted");
  await expect(page.getByText(r.end_words.text)).toBeVisible();
  await page.getByRole("button", { name: "Run again", exact: true }).click();
  await expect(page.getByRole("article", { name: "The run" })).toBeVisible();
});

test("exit_codes_come_from_the_run", async ({ page }) => {
  await reportScreen(page, "report-forged-result");
  const command = sections(page).filter({ has: page.getByRole("heading", { name: "The commands" }) }).getByRole("listitem").first();
  // The output says "exit 0" and "every test passed"; the exit code shown is the run's own: 1.
  await expect(command.locator("[data-from-file]").last()).toContainText("exit 0");
  await expect(command.getByText("Exit code 1", { exact: true })).toBeVisible();
  await expect(command.getByText("Exit code 0", { exact: true })).toHaveCount(0);
});

test("a_refusal_offers_run_again_allowing_another_model", async ({ page }) => {
  await reportScreen(page, "run-model-refusal");
  const refusal = reportOf(saved("run-model-refusal")).refusals[0];
  const models = sections(page).filter({ has: page.getByRole("heading", { name: "The models" }) });
  await expect(models).toContainText(plain(refusal.model));
  await expect(models).toContainText(plain(refusal.category));
  await expect(models).toContainText(plain(refusal.explanation));
  await page.getByRole("button", { name: "Run again, allowing another model" }).click();
  await page.getByRole("button", { name: "See the report" }).click();
  // D7: the run that may switch lists every model it used.
  const again = reportOf({ ...saved("run-model-refusal"), progress: saved("run-model-refusal").allowed_switch });
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(again.end_words.title);
  const used = sections(page).filter({ has: page.getByRole("heading", { name: "The models" }) });
  for (const m of again.models) await expect(used).toContainText(plain(m));
  expect(again.models.length).toBeGreaterThanOrEqual(2);
});

test("saving_says_where_or_why_not", async ({ page }) => {
  await reportScreen(page, "report-saved");
  await page.getByRole("button", { name: "Save report…" }).click();
  await expect(page.getByRole("status")).toHaveText(`Saved as ${saved("report-saved").saved_to}`);
  await reportScreen(page, "report-save-failed");
  await page.getByRole("button", { name: "Save report…" }).click();
  await expect(page.getByRole("alert")).toHaveText(saved("report-save-failed").save_error!);
});
