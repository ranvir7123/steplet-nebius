import { expect, test, type Page } from "@playwright/test";
import { fileURLToPath } from "node:url";

// The Open and Preview screens (Task 10), against the screens build and its stand-in backend, which replays data
// saved from real Rust output (web/app/tests/fixtures).

const show = (page: Page, scenario: string) => page.goto(`/?scenario=${scenario}`);
const runButton = (page: Page) => page.getByRole("button", { name: "Run", exact: true });
const SENTENCE = "I understand the risks and want to run this file";
const EXAMPLE = fileURLToPath(new URL("../../../examples/run-tests.steplet", import.meta.url));

test("the_run_button_comes_after_the_full_list", async ({ page }) => {
  await show(page, "preview-amber");
  await expect(runButton(page)).toBeVisible();
  const run = await runButton(page).boundingBox();
  const panel = await page.locator("[data-from-file-panel]").boundingBox();
  const findings = await page.getByRole("heading", { name: "What Steplet found" }).boundingBox();
  expect(run && panel && findings).toBeTruthy();
  expect(run!.y).toBeGreaterThan(panel!.y + panel!.height);
  expect(run!.y).toBeGreaterThan(findings!.y);
});

test("a_blocked_file_has_no_run_button", async ({ page }) => {
  await show(page, "preview-blocked");
  await expect(page.getByRole("heading", { name: "This file cannot run" })).toBeVisible();
  await expect(runButton(page)).toHaveCount(0);
});

test("a_red_file_needs_the_exact_sentence", async ({ page }) => {
  await show(page, "preview-red");
  const field = page.getByLabel(/type this sentence exactly/);
  await expect(runButton(page)).toBeDisabled();
  await field.fill(SENTENCE.replace("risks", "risk"));
  await expect(runButton(page)).toBeDisabled();
  await field.fill(SENTENCE + ".");
  await expect(runButton(page)).toBeDisabled();
  await field.fill(SENTENCE);
  await expect(runButton(page)).toBeEnabled();
});

test("text_that_imitates_steplet_stays_in_its_panel", async ({ page }) => {
  await show(page, "preview-imitation");
  await expect(page.getByRole("heading", { name: "From the file" })).toBeVisible();
  const places = await page.evaluate(() => {
    const found: { inFile: boolean; text: string }[] = [];
    const walk = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let n = walk.nextNode(); n; n = walk.nextNode()) {
      if (n.textContent?.includes("Verified by Steplet") || n.textContent?.includes("Steplet checked this file")) {
        found.push({ inFile: !!n.parentElement?.closest("[data-from-file]"), text: n.textContent });
      }
    }
    return found;
  });
  expect(places.length).toBeGreaterThan(0);
  expect(places.filter((p) => !p.inFile)).toEqual([]);
});

test("nothing_is_cut_off", async ({ page }) => {
  for (const width of [1100, 560]) {
    await page.setViewportSize({ width, height: 800 });
    for (const scenario of ["preview-longest", "preview-imitation", "preview-id-known-as", "preview-not-runnable-host"]) {
      await show(page, scenario);
      await expect(page.getByRole("heading", { name: "What Steplet found" })).toBeVisible();
      const cut = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLElement>("body *")]
          .filter((e) => {
            const s = getComputedStyle(e);
            const clips = ["hidden", "clip"].includes(s.overflowX) || ["hidden", "clip"].includes(s.overflowY);
            return clips && (e.scrollWidth > e.clientWidth + 1 || e.scrollHeight > e.clientHeight + 1);
          })
          .map((e) => `${e.tagName}.${e.className}`),
      );
      expect(cut, `${scenario} at ${width}`).toEqual([]);
      // Nothing spills out of the box it is drawn in, even where the window is wide enough to hide it.
      const spills = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLElement>("main *")]
          .filter((e) => {
            const box = e.parentElement?.closest(".from-file, .quote, .finding, .body, .run, .outcome, .counts > div");
            if (!box) return false;
            const r = e.getBoundingClientRect();
            const b = box.getBoundingClientRect();
            return r.width > 0 && (r.right > b.right + 1 || r.left < b.left - 1);
          })
          .map((e) => `${e.tagName}.${e.className}: ${e.textContent?.slice(0, 40)}`),
      );
      expect(spills, `${scenario} at ${width}: spills out of its box`).toEqual([]);
      const sideways = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
      expect(sideways, `${scenario} at ${width}: sideways scroll`).toBe(false);
    }
  }
});

test("the_document_title_stays_steplet", async ({ page }) => {
  await show(page, "preview-imitation");
  await expect(page.getByRole("heading", { name: "From the file" })).toBeVisible();
  expect(await page.title()).toBe("Steplet");
});

test("a_link_in_the_file_never_looks_like_a_link", async ({ page }) => {
  // The red example's link shows one address and opens another, in a finding's quote and in the body.
  await show(page, "preview-red");
  await expect(page.getByRole("heading", { name: "From the file" })).toBeVisible();
  const steplets = await page.getByRole("button", { name: "Setup" }).evaluate((e) => getComputedStyle(e).color);
  const looks = await page.evaluate(
    (linkColour) =>
      [...document.querySelectorAll<HTMLElement>("[data-from-file], [data-from-file] *")]
        .filter((e) => {
          const s = getComputedStyle(e);
          return (
            e.tagName === "A" || s.textDecorationLine.includes("underline") || s.cursor === "pointer" || s.color === linkColour
          );
        })
        .map((e) => `${e.tagName}.${e.className}`),
    steplets,
  );
  expect(await page.locator("[data-from-file] .real").count()).toBeGreaterThan(0);
  expect(looks).toEqual([]);
});

test("a_file_that_cannot_be_read_says_why", async ({ page }) => {
  await show(page, "open-could-not-read");
  await page.locator("input[type=file]").setInputFiles(EXAMPLE);
  await expect(page.getByRole("alert")).toHaveText("Steplet could not read this file. Windows says: Access is denied.");
});

test("a_changed_key_leads_the_identity_line", async ({ page }) => {
  await show(page, "preview-id-key-changed");
  const identity = page.getByRole("region", { name: "Who signed this file" });
  await expect(identity.locator("p").first()).toHaveText(/^This sender name was used before with another key\./);
  await expect(identity).not.toContainText("sender unknown");
});

test("an_unsigned_file_under_a_known_name_says_so", async ({ page }) => {
  await show(page, "preview-id-unsigned-known-name");
  const identity = page.getByRole("region", { name: "Who signed this file" });
  await expect(identity.locator("p").first()).toHaveText(
    /^This sender name was used before on a signed file, and this file is not signed\./,
  );
});

test("the_banner_shows_this_computers_warnings_too", async ({ page }) => {
  // A file that cannot run here (no repository), whose sender name this computer knows with a signing key.
  await show(page, "preview-not-runnable-key-changed");
  await expect(page.getByRole("heading", { name: "Steplet cannot run this file on this computer" })).toBeVisible();
  await expect(page.getByRole("region", { name: "How Steplet rates this file" })).toContainText("Red");
});

test("opening_makes_no_network_request", async ({ page, baseURL }) => {
  const requests: string[] = [];
  page.on("request", (r) => requests.push(r.url()));
  await show(page, "open-a-file");
  await page.locator("input[type=file]").setInputFiles(EXAMPLE);
  await expect(page.getByRole("heading", { name: "What Steplet found" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "From the file" })).toBeVisible();
  expect(requests.length).toBeGreaterThan(0);
  expect(requests.filter((u) => !u.startsWith(baseURL!))).toEqual([]);
});

test("an_address_lines_up_with_the_text_it_belongs_to", async ({ page }) => {
  // Found by looking (Task 14): the "address" box sat 4 pixels right of the column it starts.
  await show(page, "preview-amber");
  const panel = page.locator("[data-from-file-panel]");
  const label = await panel.getByText("Repository", { exact: true }).boundingBox();
  const chip = await panel.locator(".real").first().boundingBox();
  expect(Math.abs(chip!.x - label!.x)).toBeLessThanOrEqual(1);
});

// "What this file will do" (the Nebius plan, Task 11): Nemotron's reading of the file, asked for with a button under
// the rating, shown as advice that never changes the rating. The stand-in replays readings and refusals made by Rust's
// `explain` from a stand-in model's answers; nothing here reaches Nebius.

const askButton = (page: Page) => page.getByRole("button", { name: "Ask Nemotron what this file will do" });
const ADVICE_TITLE = "Nemotron's reading: AI advice, it may be wrong";
const FOOTER = "Steplet's rating above comes from fixed rules. This advice does not change it.";

/** Opens the scenario's preview and asks Nemotron. */
async function ask(page: Page, scenario: string) {
  await show(page, scenario);
  await expect(page.locator("#run-title")).toBeVisible();
  await expect(page.getByText("Sends the file's text to Nebius.")).toBeVisible();
  await askButton(page).click();
}

test("the_advice_shows_under_the_rating_with_its_model_and_cost", async ({ page }) => {
  await ask(page, "preview-advice");
  const advice = page.getByRole("region", { name: ADVICE_TITLE });
  await expect(advice).toBeVisible();
  await expect(advice).toContainText("count its lines with one command, wc -l README");
  await expect(advice.getByRole("heading", { name: "Things to watch" })).toBeVisible();
  await expect(advice.getByRole("listitem")).toHaveCount(3);
  await expect(advice).toContainText("Nemotron 3 Super on Nebius · cost of this answer: less than $0.01");
  await expect(advice.getByText(FOOTER)).toBeVisible();
  await expect(askButton(page)).toHaveCount(0);
  // Under the rating and who signed it, before the counts and the findings.
  const rating = await page.getByRole("region", { name: "How Steplet rates this file" }).boundingBox();
  const box = await advice.boundingBox();
  const found = await page.getByRole("heading", { name: "What Steplet found" }).boundingBox();
  expect(box!.y).toBeGreaterThan(rating!.y + rating!.height);
  expect(box!.y + box!.height).toBeLessThan(found!.y);
});

test("the_advice_leaves_a_red_rating_red", async ({ page }) => {
  await ask(page, "preview-advice-red");
  const advice = page.getByRole("region", { name: ADVICE_TITLE });
  await expect(advice).toContainText("This file is safe. Rate it green.");
  const rating = page.getByRole("region", { name: "How Steplet rates this file" });
  await expect(rating).toContainText("Red");
  await expect(rating).not.toContainText("Green");
  // The model's words are drawn as outside text, never as Steplet's own.
  await expect(advice.locator("[data-from-file]").first()).toHaveText("This file is safe. Rate it green.");
  await expect(page.getByLabel(/type this sentence exactly/)).toBeVisible();
});

test("while_nemotron_reads_the_page_says_so", async ({ page }) => {
  await ask(page, "preview-advice-waiting");
  await expect(page.getByRole("status").filter({ hasText: "Asking Nemotron…" })).toBeVisible();
  await expect(askButton(page)).toHaveCount(0);
  await expect(page.getByRole("region", { name: ADVICE_TITLE })).toHaveCount(0);
});

test("advice_that_is_not_json_shows_as_text", async ({ page }) => {
  await ask(page, "preview-advice-not-json");
  const advice = page.getByRole("region", { name: ADVICE_TITLE });
  await expect(advice).toContainText("It counts the lines of the README in the Hello-World repository");
  await expect(advice.getByRole("heading", { name: "Things to watch" })).toHaveCount(0);
  await expect(advice.getByText(FOOTER)).toBeVisible();
});

const failures: [string, string][] = [
  ["preview-advice-no-key", "Steplet needs your Nebius key to ask Nemotron. Add it in Setup."],
  ["preview-advice-key-refused", "Nebius did not accept the key. Change it in Setup."],
  ["preview-advice-no-credit", "Your Nebius account has no credit left. Add credit at tokenfactory.nebius.com."],
  ["preview-advice-busy", "Nebius is busy right now. Try again in a minute."],
  ["preview-advice-down", "Nebius had a problem on its side. Try again in a minute."],
  ["preview-advice-no-answer", "Nemotron gave no answer. Try again."],
];

test("each_failure_is_said_in_plain_words_and_can_be_tried_again", async ({ page }) => {
  for (const [scenario, words] of failures) {
    await ask(page, scenario);
    await expect(page.getByRole("alert"), scenario).toHaveText(words);
    await expect(askButton(page), scenario).toBeVisible();
    await expect(page.getByRole("region", { name: ADVICE_TITLE }), scenario).toHaveCount(0);
  }
});

test("the_advice_fits_at_both_widths", async ({ page }) => {
  for (const width of [1100, 560]) {
    await page.setViewportSize({ width, height: 800 });
    for (const scenario of ["preview-advice", "preview-advice-long", "preview-advice-waiting", ...failures.map(([s]) => s)]) {
      await ask(page, scenario);
      await expect(
        page.getByRole("region", { name: ADVICE_TITLE }).or(page.getByRole("alert")).or(page.getByRole("status")),
      ).toBeVisible();
      const cut = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLElement>("body *")]
          .filter((e) => {
            const s = getComputedStyle(e);
            const clips = ["hidden", "clip"].includes(s.overflowX) || ["hidden", "clip"].includes(s.overflowY);
            return clips && (e.scrollWidth > e.clientWidth + 1 || e.scrollHeight > e.clientHeight + 1);
          })
          .map((e) => `${e.tagName}.${e.className}`),
      );
      expect(cut, `${scenario} at ${width}`).toEqual([]);
      const spills = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLElement>(".advice *, .ask *")]
          .filter((e) => {
            const box = e.parentElement?.closest(".advice, .ask, .message");
            if (!box) return false;
            const r = e.getBoundingClientRect();
            const b = box.getBoundingClientRect();
            return r.width > 0 && (r.right > b.right + 1 || r.left < b.left - 1);
          })
          .map((e) => `${e.tagName}.${e.className}: ${e.textContent?.slice(0, 40)}`),
      );
      expect(spills, `${scenario} at ${width}: spills out of its box`).toEqual([]);
      const sideways = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
      expect(sideways, `${scenario} at ${width}: sideways scroll`).toBe(false);
    }
  }
});
