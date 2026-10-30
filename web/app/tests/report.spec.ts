import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
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

test("run_again_after_allowing_another_model_is_a_plain_run", async ({ page }) => {
  await reportScreen(page, "run-model-refusal");
  const f = saved("run-model-refusal");
  const refused = reportOf(f).end_words.title;
  const switched = reportOf({ ...f, progress: f.allowed_switch }).end_words.title;
  expect(switched).not.toBe(refused);
  await page.getByRole("button", { name: "Run again, allowing another model" }).click();
  await expect(page.getByRole("region", { name: switched })).toBeVisible();
  // The live log's own "Run again" does not carry the switch over: the model refuses again.
  await page.getByRole("button", { name: "Run again", exact: true }).click();
  await expect(page.getByRole("region", { name: refused })).toBeVisible();
});

test("saving_says_where_or_why_not", async ({ page }) => {
  await reportScreen(page, "report-saved");
  await page.getByRole("button", { name: "Save report…" }).click();
  await expect(page.getByRole("status")).toHaveText(`Saved as ${saved("report-saved").saved_to}`);
  await reportScreen(page, "report-save-failed");
  await page.getByRole("button", { name: "Save report…" }).click();
  await expect(page.getByRole("alert")).toHaveText(saved("report-save-failed").save_error!);
});

test("a_failed_command_stands_out_and_a_refused_one_did_not_run", async ({ page }) => {
  // Found by looking (Task 14): "Exit code 1" looked just like "Exit code 0", and a command refused before it ran
  // said "Exit code not known".
  const commands = () => sections(page).filter({ has: page.getByRole("heading", { name: "The commands" }) });
  await reportScreen(page, "report-finished");
  const passed = await commands().getByText("Exit code 0", { exact: true }).evaluate((e) => getComputedStyle(e).color);
  await reportScreen(page, "report-test-failed");
  const failed = await commands().getByText("Exit code 1", { exact: true }).evaluate((e) => getComputedStyle(e).color);
  expect(failed).not.toBe(passed);
  await reportScreen(page, "report-interrupted");
  await expect(commands().getByText("It did not run", { exact: true })).toBeVisible();
  await expect(commands().getByText("Exit code not known")).toHaveCount(0);
});

// Steplet's own agent (the Nebius plan, Task 7): the report says the model and what the run spent, says in plain words
// which limit ended the run, and has no trust check: there are no Claude Code settings to check. Pictured, dark, in
// docs/screens/nebius/task7-report-*.
const NEBIUS_SHOTS = fileURLToPath(new URL("../../../docs/screens/nebius/", import.meta.url));
const AGENT_REPORTS = ["report-agent-finished", "report-agent-spending-cap", "report-agent-step-limit", "report-agent-key-refused"];
const models = (page: Page) => sections(page).filter({ has: page.getByRole("heading", { name: "The models" }) });

test("the_report_says_the_model_and_what_was_spent", async ({ page }) => {
  for (const scenario of AGENT_REPORTS) {
    await reportScreen(page, scenario);
    const spent = reportOf(saved(scenario)).spent!;
    for (const words of [spent.model, spent.money, spent.tokens]) {
      await expect(models(page).getByText(words, { exact: true }), `${scenario}: ${words}`).toBeVisible();
    }
    await expect(page.getByRole("heading", { name: "The trust check" })).toHaveCount(0);
    await expect(page.getByText(/\bgate\b/i)).toHaveCount(0);
  }
  // Nebius refused the key before any answer: nothing was spent, and the report says so.
  const refused = reportOf(saved("report-agent-key-refused")).spent!;
  expect([refused.money, refused.tokens]).toEqual(["$0.00 of $0.25", "The model gave no answer."]);
});

test("each_limit_is_the_reports_first_line", async ({ page }) => {
  for (const scenario of ["report-agent-spending-cap", "report-agent-step-limit"]) {
    await reportScreen(page, scenario);
    const r = reportOf(saved(scenario));
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Stopped at a limit");
    await expect(page.getByText(r.end_words.text, { exact: true })).toBeVisible();
  }
});

test("a_claude_code_report_keeps_its_trust_check_and_shows_no_spending", async ({ page }) => {
  await reportScreen(page, "report-finished");
  await expect(page.getByRole("heading", { name: "The trust check" })).toBeVisible();
  await expect(page.getByText(/on Nebius/)).toHaveCount(0);
});

for (const width of [1100, 560]) {
  test(`the_agents_report_fits_at_${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height: 800 });
    await page.emulateMedia({ colorScheme: "dark" });
    for (const scenario of AGENT_REPORTS) {
      await reportScreen(page, scenario);
      const sideways = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
      expect(sideways, `${scenario}: sideways scroll`).toBe(false);
      await page.evaluate(() => document.fonts.ready);
      await page.mouse.move(width - 1, 799);
      await page.screenshot({
        path: `${NEBIUS_SHOTS}task7-${scenario}-dark-${width}.png`,
        fullPage: true,
        animations: "disabled",
      });
    }
  });
}
