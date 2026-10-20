// Refuses every way of drawing text as HTML in the screens (the plan's global constraint "File text on screen";
// smaller choice S12: a script, not a linting tool that would be a download). File text is drawn only by
// web/shared/FileText.svelte, from display segments, as plain text.
//
// Usage: node scripts/no-raw-html.mjs              scans web/app and web/shared
//        node scripts/no-raw-html.mjs <folder>...  scans those folders
//        node scripts/no-raw-html.mjs --self-test  checks that the scan finds a planted mistake
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const WEB = join(dirname(fileURLToPath(import.meta.url)), "..");

// Each way a string becomes HTML in a page: Svelte's raw-HTML tag and the browser's own.
const REFUSED = [
  "{@html",
  "innerHTML",
  "outerHTML",
  "insertAdjacentHTML",
  "document.write",
  "createContextualFragment",
  "setHTMLUnsafe",
  "parseHTMLUnsafe",
  "srcdoc",
];

const SCANNED = /\.(svelte|ts|js|mjs|cjs|html)$/;

function files(folder) {
  const out = [];
  for (const entry of readdirSync(folder, { withFileTypes: true })) {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) out.push(...files(path));
    else if (SCANNED.test(entry.name)) out.push(path);
  }
  return out;
}

/** Every refused word in the folders, as "file:line: word". */
function scan(folders) {
  const hits = [];
  for (const folder of folders) {
    for (const file of files(folder)) {
      const lines = readFileSync(file, "utf8").split("\n");
      lines.forEach((line, i) => {
        for (const word of REFUSED) {
          if (line.includes(word)) hits.push(`${relative(WEB, file)}:${i + 1}: ${word}`);
        }
      });
    }
  }
  return hits;
}

function selfTest() {
  const folder = mkdtempSync(join(tmpdir(), "steplet-no-raw-html-"));
  try {
    writeFileSync(join(folder, "Clean.svelte"), "<p>{text}</p>\n");
    if (scan([folder]).length !== 0) throw new Error("a clean file was refused");
    writeFileSync(join(folder, "Planted.svelte"), "<p>{@html x}</p>\n");
    const hits = scan([folder]);
    if (hits.length !== 1 || !hits[0].endsWith("Planted.svelte:1: {@html")) {
      throw new Error(`the planted {@html x} was not found exactly once: ${JSON.stringify(hits)}`);
    }
  } finally {
    rmSync(folder, { recursive: true, force: true });
  }
  console.log("self-test passed");
}

const args = process.argv.slice(2);
if (args[0] === "--self-test") {
  selfTest();
} else {
  const folders = args.length ? args : [join(WEB, "app"), join(WEB, "shared")];
  const hits = scan(folders);
  if (hits.length) {
    for (const hit of hits) console.error(hit);
    console.error(`raw HTML refused: ${hits.length} place(s)`);
    process.exit(1);
  }
  console.log("no raw HTML");
}
