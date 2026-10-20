import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Fixture } from "../../shared/types/Fixture";
import type { QuestionView } from "../../shared/types/QuestionView";
import type { Seg } from "../../shared/types/Seg";

// The run screen and the ask screen (Task 12), reached as a receiver reaches them: the preview, Run, then "Run with
// Claude Code". Against the screens build and its stand-in backend, which replays progress saved from real Rust
// output (web/app/tests/fixtures), at the pace web/app/lib/scenarios.ts sets.

const saved = (name: string): Fixture =>
  JSON.parse(readFileSync(new URL(`./fixtures/${name}.json`, import.meta.url), "utf8")) as Fixture;
const plain = (segs: Seg[]) => segs.map((s) => (s.kind === "text" ? s.text : "")).join("");
const YES = "Yes, allow this exact request for this run";

function questionOf(f: Fixture): QuestionView {
  for (const p of f.progress) if (p.kind === "question") return p.question;
  throw new Error("the scenario asks nothing");
}

/** Opens the scenario's file and starts its run; returns when "Run with Claude Code" was pressed. */
async function runOf(page: Page, scenario: string) {
  await page.goto(`/?scenario=${scenario}`);
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await page.getByRole("button", { name: "Run with Claude Code" }).click();
}

const log = (page: Page) => page.getByRole("log", { name: "What happens in the run" });
const atTheEnd = (page: Page) =>
  page.evaluate(() => window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 2);

test("ask_starts_on_no_and_yes_waits_one_second", async ({ page }) => {
  await runOf(page, "ask-command");
  const f = saved("ask-command");
  const dialog = page.getByRole("dialog", { name: questionOf(f).ask });
  await expect(dialog).toBeVisible();
  const shown = Date.now();
  const yes = dialog.getByRole("button", { name: YES });
  await expect(dialog.getByRole("button", { name: "No", exact: true })).toBeFocused();
  await expect(yes).toBeDisabled();
  // A click meant for something else cannot answer it: Yes works only once the question has been there a second.
  await expect(yes).toBeEnabled({ timeout: 3000 });
  expect(Date.now() - shown).toBeGreaterThanOrEqual(800);
  await expect(dialog).toContainText(/No answer in [45]:\d\d means no/);
  // The rest of the window waits behind the question.
  await expect(page.getByRole("button", { name: "Stop", exact: true }).click({ timeout: 500 })).rejects.toThrow();
  await yes.click();
  await expect(dialog).toHaveCount(0);
  await expect(log(page)).toContainText(f.replies.yes);
});

test("ask_shows_the_whole_request", async ({ page }) => {
  for (const name of ["ask-long-command", "ask-file-write"]) {
    await runOf(page, name);
    const q = questionOf(saved(name));
    const dialog = page.getByRole("dialog", { name: q.ask });
    await expect(dialog).toBeVisible();
    const texts = await dialog.locator("[data-from-file]").allTextContents();
    // Every field of the request, every character of it: a 300-character command, a 200-line file.
    for (const field of q.fields) expect(texts).toContain(plain(field.value));
    await expect(dialog).toContainText(q.reason);
    await expect(dialog).toContainText(q.holds);
  }
  expect(plain(questionOf(saved("ask-long-command")).fields[0].value).length).toBeGreaterThanOrEqual(300);
});

test("a_withdrawn_question_closes_with_a_note", async ({ page }) => {
  await runOf(page, "ask-withdrawn");
  const f = saved("ask-withdrawn");
  const dialog = page.getByRole("dialog", { name: questionOf(f).ask });
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveCount(0);
  const closed = f.progress.find((p) => p.kind === "question_closed");
  if (closed?.kind !== "question_closed") throw new Error("the scenario closes no question");
  await expect(log(page)).toContainText(closed.how);
  await expect(log(page)).toContainText(questionOf(f).ask);
});

test("the_log_follows_the_end_unless_you_scrolled_up", async ({ page }) => {
  await runOf(page, "run-40-commands");
  // Each command's output, "N README" and its line break.
  const results = log(page).getByText(/^\d+ README\n$/);
  await expect.poll(() => results.count()).toBeGreaterThanOrEqual(12);
  await expect.poll(() => atTheEnd(page)).toBe(true);
  // Scrolled up: the log stays where the receiver put it while rows keep coming.
  await page.mouse.move(500, 400);
  await page.mouse.wheel(0, -500);
  await expect.poll(() => atTheEnd(page)).toBe(false);
  const where = await page.evaluate(() => window.scrollY);
  const before = await results.count();
  await expect.poll(() => results.count()).toBeGreaterThanOrEqual(before + 4);
  expect(await page.evaluate(() => window.scrollY)).toBe(where);
  // Back to the newest, and it follows again.
  await page.getByRole("button", { name: "Show the newest" }).click();
  await expect(results).toHaveCount(40, { timeout: 15000 });
  await expect.poll(() => atTheEnd(page)).toBe(true);
});

test("stop_asks_first_and_then_says_interrupted", async ({ page }) => {
  await runOf(page, "run-stopping");
  await expect(log(page)).toContainText("rm -rf build");
  const stop = page.getByRole("button", { name: "Stop", exact: true });
  await stop.click();
  const ask = page.getByRole("dialog", { name: "Stop the run?" });
  await expect(ask).toBeVisible();
  await expect(ask.getByRole("button", { name: "Keep running" })).toBeFocused();
  await ask.getByRole("button", { name: "Keep running" }).click();
  await expect(ask).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Interrupted" })).toHaveCount(0);
  await stop.click();
  await ask.getByRole("button", { name: "Stop the run" }).click();
  await expect(page.getByRole("heading", { name: "Interrupted" })).toBeVisible();
  await expect(page.getByText("You stopped the run.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toHaveCount(0);
});

test("a_50000_line_output_keeps_the_log_responsive", async ({ page }) => {
  await runOf(page, "run-long-output");
  const started = Date.now();
  const all = log(page).getByRole("button", { name: "Show all 50,000 lines" });
  await expect(all).toBeVisible();
  expect(Date.now() - started).toBeLessThan(2000);
  const output = log(page).getByRole("region", { name: "What it printed" });
  const lines = (await output.locator("[data-from-file]").textContent())!.split("\n").filter((l) => l !== "");
  expect(lines).toHaveLength(40);
  expect(lines[0]).toBe("line 1 of the output");
  expect(lines[39]).toBe("line 40 of the output");
  // The window still answers at once.
  const asked = Date.now();
  await page.getByRole("button", { name: "Stop", exact: true }).click();
  await expect(page.getByRole("dialog", { name: "Stop the run?" })).toBeVisible();
  expect(Date.now() - asked).toBeLessThan(1000);
  await page.getByRole("button", { name: "Keep running" }).click();
  await all.click();
  await expect(output).toContainText("line 50000 of the output");
  await expect(log(page).getByRole("button", { name: "Show the first 40 lines" })).toBeVisible();
});

test("closing_the_window_during_a_run_asks_first", async ({ page }) => {
  await runOf(page, "run-stopping");
  await expect(log(page)).toContainText("rm -rf build");
  // The stand-in's way of saying Windows asked to close the window (the real one is the `close-asked` event).
  await page.evaluate(() => window.dispatchEvent(new Event("steplet-stand-in-close")));
  const ask = page.getByRole("dialog", { name: "Close Steplet?" });
  await expect(ask).toBeVisible();
  await expect(ask.getByRole("button", { name: "Keep running" })).toBeFocused();
  await ask.getByRole("button", { name: "Stop and close" }).click();
  await expect(page.getByText("Steplet is stopping the run, then it closes.")).toBeVisible();
});

test("a_wrong_key_shows_each_try_and_says_so", async ({ page }) => {
  await runOf(page, "run-wrong-key");
  // About three minutes of tries in a real run: one row, in Steplet's words, that counts them.
  const tries = log(page).getByText(/^Anthropic did not accept the key\. Claude Code tries again \(\d+ of 10\)\.$/);
  await expect(tries).toHaveCount(1);
  await expect(tries).toHaveText("Anthropic did not accept the key. Claude Code tries again (10 of 10).");
  const end = page.getByRole("region", { name: "Did not finish" });
  await expect(end).toBeVisible();
  await expect(end).toContainText("Anthropic did not accept the key. Change it in Setup.");
});
