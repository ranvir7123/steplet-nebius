import { expect, test, type Page } from "@playwright/test";
import { fileURLToPath } from "node:url";

// Every screen in every state the plan lists (Task 14, the plan's section 8), in light and dark, 1100 and 560 pixels
// wide. Each picture is saved as docs/screens/phase-4/<screen>-<state>-<light|dark>-<1100|560>.png, to be looked at.
// Each state also fails on clipped text, sideways page scroll, a control that shows no focus ring when tabbed to, and
// any request that leaves the page's own server. Against the screens build and its stand-in backend, which replays
// data saved from real Rust output (web/app/tests/fixtures).

const SHOTS = fileURLToPath(new URL("../../../docs/screens/phase-4/", import.meta.url));
const EXAMPLE = fileURLToPath(new URL("../../../examples/run-tests.steplet", import.meta.url));
const SENTENCE = "I understand the risks and want to run this file";

interface State {
  screen: string;
  state: string;
  scenario: string;
  /** From the scenario's first screen to this state, as a receiver gets there. */
  reach: (page: Page) => Promise<void>;
}

const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

/** Waits until the page stops changing: the stand-in plays a scenario's progress over a few moments. */
async function settled(page: Page) {
  let last = "";
  let same = 0;
  await expect
    .poll(
      async () => {
        const now = await page.evaluate(() => document.body.innerText.replace(/\d/g, "0"));
        same = now === last ? same + 1 : 0;
        last = now;
        return same;
      },
      { intervals: [150], timeout: 20_000 },
    )
    .toBeGreaterThanOrEqual(3);
}

async function open(page: Page) {
  await expect(page.getByRole("heading", { name: "Open a .steplet file to see what it asks for." })).toBeVisible();
}

async function preview(page: Page) {
  await expect(page.locator("#run-title")).toBeVisible();
}

async function chooser(page: Page) {
  await preview(page);
  const sentence = page.getByLabel(/type this sentence exactly/);
  if (await sentence.count()) await sentence.fill(SENTENCE);
  await button(page, "Run").click();
  await expect(page.getByRole("heading", { name: "Choose how it runs" })).toBeVisible();
}

async function setup(page: Page) {
  await button(page, "Setup").click();
  await expect(page.getByRole("heading", { name: "Set up Steplet" })).toBeVisible();
  await settled(page);
}

async function run(page: Page) {
  await chooser(page);
  await button(page, "Run with Claude Code").click();
  await expect(page.getByRole("article", { name: "The run" })).toBeVisible();
  await settled(page);
}

async function ask(page: Page) {
  await chooser(page);
  await button(page, "Run with Claude Code").click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  // Yes waits a second before it can be pressed; the picture shows the question as it then stays.
  await expect(dialog.getByRole("button", { name: /^Yes/ })).toBeEnabled({ timeout: 3000 });
  await settled(page);
}

async function report(page: Page) {
  await run(page);
  await button(page, "See the report").click();
  await expect(page.getByRole("article", { name: "The report" })).toBeVisible();
}

const states: State[] = [
  { screen: "open", state: "empty", scenario: "open-empty", reach: open },
  {
    screen: "open",
    state: "reading",
    scenario: "open-reading",
    reach: async (page) => {
      await open(page);
      await page.locator("input[type=file]").setInputFiles(EXAMPLE);
      await expect(page.getByText("Reading the file…")).toBeVisible();
    },
  },
  {
    screen: "open",
    state: "could-not-read",
    scenario: "open-could-not-read",
    reach: async (page) => {
      await open(page);
      await page.locator("input[type=file]").setInputFiles(EXAMPLE);
      await expect(page.getByRole("alert")).toBeVisible();
    },
  },
  {
    screen: "open",
    state: "crash-cleared",
    scenario: "open-crash-cleared",
    reach: async (page) => {
      await open(page);
      await expect(page.getByRole("status").first()).toBeVisible();
    },
  },
  ...[
    "green",
    "amber",
    "red",
    "blocked",
    "header-unreadable",
    "longest",
    "imitation",
    "id-unsigned",
    "id-new-key",
    "id-same-key",
    "id-known-as",
    "id-key-changed",
    "id-unsigned-known-name",
    "not-runnable-host",
    "not-runnable-no-repo",
    "not-runnable-key-changed",
  ].map((state) => ({ screen: "preview", state, scenario: `preview-${state}`, reach: preview })),
  ...["ready-no-sites", "ready-15-sites", "needs-setup", "wsl-missing"].map((state) => ({
    screen: "chooser",
    state,
    scenario: `chooser-${state}`,
    reach: chooser,
  })),
  ...["nothing-done", "wsl-missing", "wsl-too-old", "could-not-look", "all-ready", "key-saved", "switch-broken"].map(
    (state) => ({ screen: "setup", state, scenario: `setup-${state}`, reach: setup }),
  ),
  {
    screen: "setup",
    state: "installing",
    scenario: "setup-installing",
    reach: async (page) => {
      await setup(page);
      await button(page, "Install Steplet's Linux").click();
      await expect(page.getByText("Steplet is changing its Linux. This can take several minutes…")).toBeVisible();
      await settled(page);
    },
  },
  {
    screen: "setup",
    state: "install-failed",
    scenario: "setup-install-failed",
    reach: async (page) => {
      await setup(page);
      await button(page, "Install Steplet's Linux").click();
      await expect(button(page, "Start over")).toBeVisible();
      await settled(page);
    },
  },
  {
    screen: "setup",
    state: "start-over",
    scenario: "setup-install-failed",
    reach: async (page) => {
      await setup(page);
      await button(page, "Install Steplet's Linux").click();
      await button(page, "Start over").click();
      await expect(button(page, "Remove Steplet's Linux")).toBeVisible();
    },
  },
  {
    screen: "setup",
    state: "switch-fails",
    scenario: "setup-switch-fails",
    reach: async (page) => {
      await setup(page);
      await page.getByLabel("Open .steplet files with Steplet").click();
      await expect(page.getByRole("alert")).toBeVisible();
    },
  },
  ...["fetching", "ready-removed", "fetch-failed", "commit-missing"].map((state) => ({
    screen: "prepare",
    state,
    scenario: `prepare-${state}`,
    reach: run,
  })),
  ...["started", "busy", "long-command", "40-commands", "tripwire", "model-refusal", "wrong-key"].map((state) => ({
    screen: "run",
    state,
    scenario: `run-${state}`,
    reach: run,
  })),
  { screen: "run", state: "long-output-closed", scenario: "run-long-output", reach: run },
  {
    screen: "run",
    state: "long-output-open",
    scenario: "run-long-output",
    reach: async (page) => {
      await run(page);
      await button(page, "Show all 50,000 lines").click();
      await expect(button(page, "Show the first 40 lines")).toBeVisible();
    },
  },
  {
    screen: "run",
    state: "stop-question",
    scenario: "run-stopping",
    reach: async (page) => {
      await run(page);
      await button(page, "Stop").click();
      await expect(page.getByRole("dialog", { name: "Stop the run?" })).toBeVisible();
    },
  },
  {
    screen: "run",
    state: "stopping",
    scenario: "run-stopping",
    reach: async (page) => {
      await run(page);
      await button(page, "Stop").click();
      await button(page, "Stop the run").click();
      await expect(page.getByText("Stopping…")).toBeVisible();
    },
  },
  {
    screen: "run",
    state: "stopped",
    scenario: "run-stopping",
    reach: async (page) => {
      await run(page);
      await button(page, "Stop").click();
      await button(page, "Stop the run").click();
      await expect(page.getByRole("heading", { name: "Interrupted" })).toBeVisible();
      await settled(page);
    },
  },
  {
    screen: "run",
    state: "close-question",
    scenario: "run-stopping",
    reach: async (page) => {
      await run(page);
      await page.evaluate(() => window.dispatchEvent(new Event("steplet-stand-in-close")));
      await expect(page.getByRole("dialog", { name: "Close Steplet?" })).toBeVisible();
    },
  },
  ...["command", "file-write", "long-command", "web-address", "10-seconds-left", "2-more-waiting"].map((state) => ({
    screen: "ask",
    state,
    scenario: `ask-${state}`,
    reach: ask,
  })),
  {
    screen: "ask",
    state: "withdrawn",
    scenario: "ask-withdrawn",
    reach: async (page) => {
      await chooser(page);
      await button(page, "Run with Claude Code").click();
      await expect(page.getByRole("dialog")).toBeVisible();
      await expect(page.getByRole("dialog")).toHaveCount(0);
      await settled(page);
    },
  },
  {
    screen: "ask",
    state: "answer-fails",
    scenario: "ask-answer-fails",
    reach: async (page) => {
      await ask(page);
      await page.getByRole("dialog").getByRole("button", { name: "No", exact: true }).click();
      await expect(page.getByRole("dialog").getByRole("alert")).toBeVisible();
    },
  },
  ...[
    "finished",
    "test-failed",
    "tripwire",
    "interrupted",
    "faults",
    "40-commands",
    "forged-result",
  ].map((state) => ({ screen: "report", state, scenario: `report-${state}`, reach: report })),
  { screen: "report", state: "model-refusal", scenario: "run-model-refusal", reach: report },
  {
    screen: "report",
    state: "saved",
    scenario: "report-saved",
    reach: async (page) => {
      await report(page);
      await button(page, "Save report…").click();
      await expect(page.getByRole("status")).toBeVisible();
    },
  },
  {
    screen: "report",
    state: "save-failed",
    scenario: "report-save-failed",
    reach: async (page) => {
      await report(page);
      await button(page, "Save report…").click();
      await expect(page.getByRole("alert")).toBeVisible();
    },
  },
];

/** Text cut off by a box that hides what does not fit (Task 10's check), on any screen. */
async function clipped(page: Page) {
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>("body *")]
      .filter((e) => {
        const s = getComputedStyle(e);
        const clips = ["hidden", "clip"].includes(s.overflowX) || ["hidden", "clip"].includes(s.overflowY);
        return clips && (e.scrollWidth > e.clientWidth + 1 || e.scrollHeight > e.clientHeight + 1);
      })
      .map((e) => `${e.tagName}.${e.className}`),
  );
}

/** Tabs through every control once; returns the ones that show no focus ring. */
async function withoutFocusRing(page: Page) {
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const seen = new Set<number>();
  const without: string[] = [];
  let outside = 0;
  for (let i = 0; i < 300 && outside < 2; i++) {
    await page.keyboard.press("Tab");
    const f = await page.evaluate(() => {
      const e = document.activeElement as HTMLElement | null;
      if (!e || e === document.body) return null;
      const s = getComputedStyle(e);
      return {
        at: [...document.querySelectorAll("*")].indexOf(e),
        name: `${e.tagName.toLowerCase()} "${(e.getAttribute("aria-label") ?? e.textContent ?? "").trim().slice(0, 40)}"`,
        ring: s.outlineStyle !== "none" && parseFloat(s.outlineWidth) >= 2,
      };
    });
    if (!f) {
      outside++;
      continue;
    }
    outside = 0;
    if (seen.has(f.at)) break;
    seen.add(f.at);
    if (!f.ring) without.push(f.name);
  }
  return without;
}

for (const s of states) {
  for (const theme of ["light", "dark"] as const) {
    for (const width of [1100, 560]) {
      const name = `${s.screen}-${s.state}-${theme}-${width}`;
      test(name, async ({ page, baseURL }) => {
        const outside: string[] = [];
        page.on("request", (r) => {
          const url = r.url();
          if (!url.startsWith(baseURL!) && !url.startsWith("data:") && !url.startsWith("blob:")) outside.push(url);
        });
        await page.setViewportSize({ width, height: 800 });
        await page.emulateMedia({ colorScheme: theme });
        await page.goto(`/?scenario=${s.scenario}`);
        await s.reach(page);
        await page.evaluate(() => document.fonts.ready);
        // The mouse rests where the last button was pressed: away from the page, so no picture shows a hover.
        await page.mouse.move(width - 1, 799);
        // A page the log scrolled to its newest row is pictured as the window shows it: a whole-page picture would
        // draw the bar that stays at the top of the window in the middle of the page.
        const scrolled = await page.evaluate(() => window.scrollY > 0);
        await page.screenshot({ path: `${SHOTS}${name}.png`, fullPage: !scrolled, animations: "disabled" });

        expect(await clipped(page), "text cut off").toEqual([]);
        const sideways = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
        expect(sideways, "sideways scroll").toBe(false);
        expect(await withoutFocusRing(page), "controls without a focus ring").toEqual([]);
        expect(outside, "requests that leave the page's server").toEqual([]);
      });
    }
  }
}
