// Checks that every pair of colours the screens put together is readable, in the light and the dark theme
// (the plan's UI rule: text at least 4.5 : 1; focus rings and control borders at least 3 : 1, WCAG 2.2's
// 1.4.3 and 1.4.11). The colours come from web/shared/tokens.css; the pairs are listed below.
//
// Usage: node scripts/contrast.mjs               checks web/shared/tokens.css
//        node scripts/contrast.mjs --list        the same, printing every pair's ratio
//        node scripts/contrast.mjs <file.css>    checks that file
//        node scripts/contrast.mjs --self-test   checks that a low pair is refused
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const TOKENS = join(dirname(fileURLToPath(import.meta.url)), "..", "shared", "tokens.css");

const SURFACES = ["bg", "surface", "surface-sunken", "file-bg"];
const ON_SURFACES = ["text", "text-muted", "accent-text", "file-label"];

/** [foreground, background, least ratio] */
const PAIRS = [
  ...ON_SURFACES.flatMap((fg) => SURFACES.map((bg) => [fg, bg, 4.5])),
  ["on-accent", "accent", 4.5],
  ["on-accent", "accent-hover", 4.5],
  ...["green", "amber", "red", "blocked"].map((o) => [`${o}-fg`, `${o}-bg`, 4.5]),
  ["badge-fg", "badge-bg", 4.5],
  ...SURFACES.map((bg) => ["focus", bg, 3]),
  ...SURFACES.map((bg) => ["control-border", bg, 3]),
];

/** The custom properties of each theme block: the first `:root { }` is light, the one inside the dark media query is dark. */
function themes(css) {
  const props = (block) => {
    const out = new Map();
    for (const m of block.matchAll(/--([a-z0-9-]+)\s*:\s*([^;]+);/g)) out.set(m[1], m[2].trim());
    return out;
  };
  const dark = css.match(/@media\s*\(prefers-color-scheme:\s*dark\)\s*\{\s*:root\s*\{([^}]*)\}/);
  const light = css.replace(/@media[^{]*\{\s*:root\s*\{[^}]*\}\s*\}/g, "").match(/:root\s*\{([^}]*)\}/);
  if (!light || !dark) throw new Error("tokens.css needs a :root block and a dark :root block");
  return { light: props(light[1]), dark: props(dark[1]) };
}

function luminance(hex) {
  const m = /^#([0-9a-f]{6})$/i.exec(hex);
  if (!m) throw new Error(`not a #rrggbb colour: ${hex}`);
  const [r, g, b] = [0, 2, 4].map((i) => {
    const c = parseInt(m[1].slice(i, i + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function ratio(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

/** Every problem in the file, as lines; an empty list means every pair is readable. */
function check(css, print) {
  const problems = [];
  const t = themes(css);
  for (const name of new Set([...t.light.keys(), ...t.dark.keys()])) {
    if (!t.light.has(name) || !t.dark.has(name)) problems.push(`--${name} is not set in both themes`);
  }
  for (const [theme, colours] of Object.entries(t)) {
    for (const [fg, bg, least] of PAIRS) {
      if (!colours.has(fg) || !colours.has(bg)) {
        problems.push(`${theme}: --${fg} or --${bg} is missing`);
        continue;
      }
      const r = ratio(colours.get(fg), colours.get(bg));
      const line = `${theme.padEnd(5)} --${fg} on --${bg}: ${r.toFixed(2)} : 1 (needs ${least})`;
      if (r < least) problems.push(`${line}  <-- too low`);
      else if (print) console.log(line);
    }
  }
  return problems;
}

function selfTest() {
  const good = readFileSync(TOKENS, "utf8");
  if (check(good, false).length) throw new Error("the real tokens fail, so the self-test cannot start");
  const folder = mkdtempSync(join(tmpdir(), "steplet-contrast-"));
  try {
    // Plant one low pair: muted text as light as the panel behind it.
    const low = good.replace(/--text-muted:\s*#[0-9a-f]{6};/i, "--text-muted: #eeeeee;");
    if (low === good) throw new Error("could not plant a low pair");
    writeFileSync(join(folder, "low.css"), low);
    const problems = check(readFileSync(join(folder, "low.css"), "utf8"), false);
    if (!problems.some((p) => p.startsWith("light --text-muted on --surface:"))) {
      throw new Error(`the planted low pair was not refused: ${JSON.stringify(problems)}`);
    }
  } finally {
    rmSync(folder, { recursive: true, force: true });
  }
  console.log("self-test passed");
}

const arg = process.argv[2];
if (arg === "--self-test") {
  selfTest();
} else {
  const list = arg === "--list";
  const file = arg && !list ? arg : TOKENS;
  const problems = check(readFileSync(file, "utf8"), list);
  if (problems.length) {
    for (const p of problems) console.error(p);
    process.exit(1);
  }
  console.log(`every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): ${PAIRS.length * 2} pairs`);
}
