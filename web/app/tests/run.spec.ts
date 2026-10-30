import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { Fixture } from "../../shared/types/Fixture";
import type { QuestionView } from "../../shared/types/QuestionView";
import type { Seg } from "../../shared/types/Seg";
import type { SpendView } from "../../shared/types/SpendView";

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

test("showing_all_lines_keeps_them_in_a_box_of_their_own", async ({ page }) => {
  // Found by looking (Task 14): "Show all" made the page 50,000 lines long, so the window went to line 50,000 and
  // the button that puts them away was at the far end of them.
  await runOf(page, "run-long-output");
  await log(page).getByRole("button", { name: "Show all 50,000 lines" }).click();
  const output = log(page).getByRole("region", { name: "What it printed" });
  await expect(output).toContainText("line 50000 of the output");
  await expect(log(page).getByRole("button", { name: "Show the first 40 lines" })).toBeInViewport();
  // The page grows by less than a window; the lines scroll in their own box, from its first line.
  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBeLessThan(2 * 800);
  expect(await output.evaluate((e) => e.scrollTop)).toBe(0);
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

test("an_answer_that_did_not_reach_steplet_can_be_given_again", async ({ page }) => {
  await runOf(page, "ask-answer-fails");
  const f = saved("ask-answer-fails");
  const dialog = page.getByRole("dialog", { name: questionOf(f).ask });
  await expect(dialog).toBeVisible();
  const no = dialog.getByRole("button", { name: "No", exact: true });
  await no.click();
  await expect(dialog.getByRole("alert")).toHaveText("Steplet could not pass on your answer. Try again.");
  await expect(no).toBeEnabled();
  await no.click();
  await expect(dialog).toHaveCount(0);
  await expect(log(page)).toContainText(f.replies.no);
});

test("the_live_log_says_how_each_call_ended_as_the_report_does", async ({ page }) => {
  // Task 14 review: the live log had words of its own ("Done", "Ended with an error") and hid how a refused call
  // ended; it now shows Rust's words, the report's.
  await runOf(page, "run-busy");
  const rm = log(page).locator("li").filter({ hasText: "rm -rf build" });
  await expect(rm).toContainText("It did not run");
  const wc = log(page).locator("li").filter({ hasText: "wc -l README" }).first();
  await expect(wc).toContainText("Exit code 0");
});

test("a_link_box_keeps_a_gap_from_the_text_around_it", async ({ page }) => {
  // Task 14 review: the box had a gap only after a link's shown text, so an address right after a quote touched it.
  await runOf(page, "prepare-fetch-failed");
  const gaps = await page.getByRole("alert").locator(".real").first().evaluate((box) => {
    const link = box.parentElement!;
    const edge = (node: Node | null, end: boolean) => {
      if (!node || node.nodeType !== Node.TEXT_NODE || !node.textContent) return null;
      const r = document.createRange();
      const at = end ? node.textContent.length - 1 : 0;
      r.setStart(node, at);
      r.setEnd(node, at + 1);
      return r.getBoundingClientRect();
    };
    const b = box.getBoundingClientRect();
    // The nearest text on each side, past the markers Svelte leaves between pieces.
    const near = (from: Node | null, back: boolean) => {
      let n = from;
      while (n && (n.nodeType === Node.COMMENT_NODE || (n.nodeType === Node.TEXT_NODE && !n.textContent))) {
        n = back ? n.previousSibling : n.nextSibling;
      }
      return n;
    };
    const before = edge(near(link.previousSibling, true), true);
    const after = edge(near(link.nextSibling, false), false);
    return { before: before ? b.left - before.right : null, after: after ? after.left - b.right : null };
  });
  expect(gaps.before).toBeGreaterThanOrEqual(3);
  expect(gaps.after).toBeGreaterThanOrEqual(3);
});

// Steplet's own agent (the Nebius plan, Task 7): the bar at the top of the run says the model, what the run has spent
// of its cap and the step it is on, at both widths and whatever the model's name. Pictured, dark, in
// docs/screens/nebius/task7-run-*.
const NEBIUS_SHOTS = fileURLToPath(new URL("../../../docs/screens/nebius/", import.meta.url));

/** The last of the scenario's spending, as the top line ends up showing it. */
function spendOf(f: Fixture): SpendView {
  let last: SpendView | null = null;
  for (const p of f.progress) if (p.kind === "spent") last = p.spent;
  if (!last) throw new Error("the scenario spends nothing");
  return last;
}
const topLine = (s: SpendView) => `${s.model} · ${s.money} · ${s.steps}`;

/** The end panel's sentence for a run that stopped at a limit. */
function limitOf(f: Fixture): string {
  for (const p of f.progress) if (p.kind === "finished" && p.report.end.kind === "stopped_at_limit") return p.report.end.why;
  throw new Error("the scenario did not stop at a limit");
}

for (const width of [1100, 560]) {
  test(`the_top_line_says_the_model_the_spend_and_the_step_at_${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height: 800 });
    await page.emulateMedia({ colorScheme: "dark" });
    for (const [scenario, ends] of [
      ["run-agent-long-model", false],
      ["run-agent-spending-cap", true],
      ["run-agent-step-limit", true],
      ["run-agent-key-refused", true],
    ] as const) {
      await runOf(page, scenario);
      const f = saved(scenario);
      const line = page.getByText(topLine(spendOf(f)), { exact: true });
      await expect(line).toBeVisible({ timeout: 20_000 });
      if (ends) await expect(page.getByRole("button", { name: "See the report" })).toBeVisible();
      else await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeVisible();
      // It stays at the top of the window while the log scrolls under it, and fits the window's width.
      await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
      const box = (await line.boundingBox())!;
      expect(box.y, `${scenario}: the top line stays at the top`).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height, `${scenario}: the top line stays at the top`).toBeLessThan(200);
      expect(box.x + box.width, `${scenario}: the top line fits`).toBeLessThanOrEqual(width);
      const sideways = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
      expect(sideways, `${scenario}: sideways scroll`).toBe(false);
      // The money and the step are never broken across two lines, however long the model's name.
      for (const part of [spendOf(f).money, spendOf(f).steps]) {
        const tall = await line.getByText(part, { exact: true }).evaluate((e) => {
          const r = document.createRange();
          r.selectNodeContents(e);
          return r.getClientRects().length;
        });
        expect(tall, `${scenario}: "${part}" on one line`).toBe(1);
      }
      // A dot the receiver can see always stands between two pieces on one line: no line starts with one.
      const start = await line.getByText(spendOf(f).model, { exact: true }).evaluate((e) => {
        const r = document.createRange();
        r.selectNodeContents(e);
        return r.getClientRects()[0].left;
      });
      for (const dot of await line.getByText("·", { exact: true }).all()) {
        const at = await dot.evaluate((e) => {
          const r = e.getBoundingClientRect();
          const seen = e.contains(document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2));
          return { seen, left: r.left };
        });
        if (at.seen) expect(at.left, `${scenario}: a line starts with a dot`).toBeGreaterThan(start + 1);
      }
      await page.evaluate(() => document.fonts.ready);
      await page.mouse.move(width - 1, 799);
      await page.screenshot({ path: `${NEBIUS_SHOTS}task7-${scenario}-dark-${width}.png`, animations: "disabled" });
    }
  });
}

test("the_top_line_counts_the_steps_as_the_answers_come", async ({ page }) => {
  await runOf(page, "run-agent-step-limit");
  // The stand-in plays this run over a few seconds (scenarios.ts): an early step shows before the last one.
  await expect(page.getByText(/^Nemotron 3 Super on Nebius · .+ · step ([1-9]|[12]\d) of 40$/)).toBeVisible();
  await expect(page.getByText(topLine(spendOf(saved("run-agent-step-limit"))), { exact: true })).toBeVisible({
    timeout: 20_000,
  });
});

test("a_run_stopped_at_a_limit_says_which_in_plain_words", async ({ page }) => {
  const cap = saved("run-agent-spending-cap");
  expect(limitOf(cap)).toMatch(
    /^Stopped before the next model call, which could have taken spending past the \$0\.25 cap\. Spent: \$0\.\d\d\.$/,
  );
  await runOf(page, "run-agent-spending-cap");
  const end = page.getByRole("region", { name: "Stopped at a limit" });
  await expect(end).toContainText(limitOf(cap));
  const steps = saved("run-agent-step-limit");
  expect(limitOf(steps)).toBe("Stopped at the step limit (40 model turns). The task may be unfinished.");
  await runOf(page, "run-agent-step-limit");
  await expect(page.getByRole("region", { name: "Stopped at a limit" })).toContainText(limitOf(steps), {
    timeout: 20_000,
  });
  // The row that opens the log names Steplet's agent and the model it asked for, not Claude Code.
  await expect(log(page).getByRole("listitem").first()).toHaveText(
    "Steplet's agent started, with the model nvidia/nemotron-3-super-120b-a12b.",
  );
  await expect(page.getByText(/\bgate\b/i)).toHaveCount(0);
});

test("a_claude_code_run_shows_no_spending", async ({ page }) => {
  await runOf(page, "run-started");
  await expect(log(page).getByRole("listitem").first()).toContainText("Claude Code started, with the model");
  await expect(page.getByText(/on Nebius/)).toHaveCount(0);
});
