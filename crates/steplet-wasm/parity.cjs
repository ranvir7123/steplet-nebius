// Runs the WASM build of the file core on every corpus file and compares its JSON with the expected answer next to
// the file, which the native build gives (cargo test -p steplet-core --test corpus). Both equal to the expected
// answers means the native and WASM builds agree byte for byte (milestone plan, phase 2).
// Usage: node crates/steplet-wasm/parity.cjs <folder made by wasm-bindgen --target nodejs> <corpus folder>
"use strict";
const fs = require("node:fs");
const path = require("node:path");

const { check } = require(path.resolve(process.argv[2], "steplet_wasm.js"));
const corpus = path.resolve(process.argv[3]);
const NOW = 1791720000n; // 2026-10-11 12:00 UTC, the corpus's fixed "now"

const EXPECTED = 134; // 62 ordinary + 72 hostile (Task 10); a missing file must not pass silently

let files = 0;
let same = 0;
for (const dir of ["ordinary", "hostile"]) {
  for (const name of fs.readdirSync(path.join(corpus, dir)).filter((n) => n.endsWith(".steplet")).sort()) {
    const file = path.join(corpus, dir, name);
    const want = fs.readFileSync(file.replace(/\.steplet$/, ".json"), "utf8");
    files += 1;
    let got;
    try {
      got = check(fs.readFileSync(file), NOW);
    } catch (e) {
      // A panic inside the WASM build throws here; name the file instead of stopping the script.
      console.log(`different: ${dir}/${name} (threw ${e})`);
      continue;
    }
    if (got === want) {
      same += 1;
    } else {
      console.log(`different: ${dir}/${name}`);
    }
  }
}
console.log(`${files} files; ${same} identical; ${files - same} different`);
if (files !== EXPECTED) {
  console.log(`expected ${EXPECTED} files`);
}
// Set the exit code and let the script end, so the lines above are flushed even when stdout is a pipe.
process.exitCode = files === EXPECTED && same === files ? 0 : 1;
