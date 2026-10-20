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
  await expect(page.getByRole("alert")).toHaveText("Steplet could not read this file: Windows says access is denied.");
});

test("a_changed_key_leads_the_identity_line", async ({ page }) => {
  await show(page, "preview-id-key-changed");
  const identity = page.getByRole("region", { name: "Who signed this file" });
  await expect(identity.locator("p").first()).toHaveText(/^This sender name was used before with another key\./);
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
