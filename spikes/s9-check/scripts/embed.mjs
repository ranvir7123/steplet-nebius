// Embeds the WASM and the test vector into source files, so the page loads nothing over the network.
import { readFileSync, writeFileSync } from "node:fs";

const wasm = readFileSync("src/wasm/s9_wasm_bg.wasm");
writeFileSync("src/wasm/s9_wasm_base64.ts", `export const WASM_BASE64 = "${wasm.toString("base64")}";\n`);
const msg = readFileSync("../s9-core/tests/vectors/msg.txt", "utf8");
const sig = readFileSync("../s9-core/tests/vectors/msg.txt.sig", "utf8");
writeFileSync("src/sample.ts", `export const SAMPLE_MESSAGE = ${JSON.stringify(msg)};\nexport const SAMPLE_SIGNATURE = ${JSON.stringify(sig)};\n`);
console.log(`embedded ${wasm.length} bytes of WASM and the test vector`);
