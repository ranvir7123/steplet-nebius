import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Fixture } from "../../shared/types/Fixture";
import type { Runner } from "../../shared/types/Runner";

// The run chooser (Task 11), reached as a receiver reaches it: from the preview, by pressing Run. Against the screens
// build and its stand-in backend, which replays data saved from real Rust output (web/app/tests/fixtures).

const saved = (name: string): Fixture =>
  JSON.parse(readFileSync(new URL(`./fixtures/${name}.json`, import.meta.url), "utf8")) as Fixture;
const SENTENCE = "I understand the risks and want to run this file";
const SUPER = "nvidia/nemotron-3-super-120b-a12b";
const NANO = "nvidia/NVIDIA-Nemotron-3-Nano-30B-A3B";
const AGENT = "Steplet's agent";
const CLAUDE = "Claude Code, in Steplet's own Linux";

async function chooserOf(page: Page, scenario: string) {
  await page.goto(`/?scenario=${scenario}`);
  const sentence = page.getByLabel(/type this sentence exactly/);
  if (await sentence.count()) await sentence.fill(SENTENCE);
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Choose how it runs" })).toBeVisible();
}

/** What the stand-in backend was asked to start (fake.ts). */
const started = (page: Page) =>
  page.evaluate(
    () =>
      (window as unknown as { stepletStarted?: { runner: Runner; typed: string | null }[] }).stepletStarted ?? [],
  );

test("codex_is_shown_unavailable_with_its_reason", async ({ page }) => {
  await chooserOf(page, "chooser-ready-no-sites");
  const codex = page.getByRole("region", { name: "Codex" });
  await expect(codex).toContainText(saved("chooser-ready-no-sites").chooser!.codex_reason);
  await expect(codex.getByRole("button")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Run with Claude Code" })).toBeEnabled();
});

test("only_the_lines_promises_returns_are_shown", async ({ page }) => {
  await chooserOf(page, "chooser-ready-15-sites");
  for (const r of saved("chooser-ready-15-sites").chooser!.runners) {
    const card = page.getByRole("region", { name: r.name, exact: true });
    const lines = card.getByRole("list", { name: "What Steplet makes sure of in this run" }).getByRole("listitem");
    // Steplet's agent's limits come first, as a tested line of their own.
    const shown = r.limits ? [r.limits, ...r.promises] : r.promises;
    await expect(lines).toHaveCount(shown.length);
    for (const [i, p] of shown.entries()) {
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
  }
});

test("a_runner_that_is_not_set_up_sends_you_to_setup", async ({ page }) => {
  await chooserOf(page, "chooser-needs-setup");
  await expect(page.getByRole("button", { name: /^Run with/ })).toHaveCount(0);
  for (const name of [AGENT, CLAUDE]) {
    await expect(page.getByRole("region", { name, exact: true })).toContainText(
      "Steplet's own Linux is not ready yet. Setup shows what is missing.",
    );
  }
  await page.getByRole("region", { name: CLAUDE, exact: true }).getByRole("button", { name: "Set up first" }).click();
  await expect(page.getByRole("heading", { name: "Set up Steplet" })).toBeVisible();
});

// The Nebius plan, Task 8.
test("chooser_offers_steplets_agent_first", async ({ page }) => {
  await chooserOf(page, "chooser-ready-no-sites");
  await expect(page.locator("section h2")).toHaveText([AGENT, CLAUDE, "Codex"]);
  const agent = page.getByRole("region", { name: AGENT, exact: true });
  await expect(agent).toContainText("Nemotron 3 Super, on Nebius Token Factory");
  await expect(agent).toContainText("$0.30 per million tokens read, $0.90 per million written");
  await expect(agent).toContainText("Stops at $0.25 or 40 steps.");
  await expect(agent).toContainText("Anything the agent leaves running is ended before its next move.");
  // The key line, shown once its tests passed (Task 9).
  await expect(agent).toContainText(
    "Your Nebius key stays on Windows: it never goes into Steplet's Linux, the report or the logs.",
  );
  // One model on offer: shown, with nothing to choose.
  await expect(agent.getByRole("radio")).toHaveCount(0);
  // Steplet's agent is the main way to run; Claude Code is the other.
  await expect(agent.getByRole("button", { name: "Run with Steplet's agent" })).toHaveClass(/primary/);
  await expect(page.getByRole("button", { name: "Run with Claude Code" })).not.toHaveClass(/primary/);
  await agent.getByRole("button", { name: "Run with Steplet's agent" }).click();
  await expect(page.getByRole("article", { name: "The run" })).toBeVisible();
  expect(await started(page)).toEqual([{ runner: { kind: "own", model: SUPER }, typed: null }]);
});

test("the_claude_code_runner_still_starts", async ({ page }) => {
  await chooserOf(page, "chooser-ready-no-sites");
  await page.getByRole("button", { name: "Run with Claude Code" }).click();
  await expect(page.getByRole("article", { name: "The run" })).toBeVisible();
  expect(await started(page)).toEqual([{ runner: { kind: "claude_code" }, typed: null }]);
});

test("the_agent_card_says_what_it_needs_without_a_key", async ({ page }) => {
  await chooserOf(page, "chooser-no-keys");
  const agent = page.getByRole("region", { name: AGENT, exact: true });
  const claude = page.getByRole("region", { name: CLAUDE, exact: true });
  await expect(agent).toContainText("Needs a Nebius key. Add it in Setup.");
  await expect(claude).toContainText("Needs an Anthropic key. Add it in Setup.");
  await expect(page.getByRole("button", { name: /^Run with/ })).toHaveCount(0);
  await agent.getByRole("button", { name: "Set up first" }).click();
  await expect(page.getByRole("heading", { name: "Set up Steplet" })).toBeVisible();
});

test("the_red_sentence_goes_with_either_runner", async ({ page }) => {
  for (const [button, runner] of [
    ["Run with Steplet's agent", { kind: "own", model: SUPER }],
    ["Run with Claude Code", { kind: "claude_code" }],
  ] as const) {
    await chooserOf(page, "chooser-red");
    await page.getByRole("button", { name: button }).click();
    await expect(page.getByRole("article", { name: "The run" })).toBeVisible();
    expect(await started(page)).toEqual([{ runner, typed: SENTENCE }]);
  }
});

test("choosing_a_model_changes_what_runs", async ({ page }) => {
  await chooserOf(page, "chooser-longest-model");
  const agent = page.getByRole("region", { name: AGENT, exact: true });
  const models = agent.getByRole("radio");
  await expect(models).toHaveCount(3);
  await expect(agent.getByRole("radio", { name: /Nemotron 3 Super/ })).toBeChecked();
  await agent.getByRole("radio", { name: /Nemotron 3 Nano/ }).check();
  await expect(agent).toContainText("Nemotron 3 Nano, on Nebius Token Factory");
  await expect(agent).toContainText(NANO);
  await agent.getByRole("button", { name: "Run with Steplet's agent" }).click();
  await expect(page.getByRole("article", { name: "The run" })).toBeVisible();
  expect(await started(page)).toEqual([{ runner: { kind: "own", model: NANO }, typed: null }]);
});
