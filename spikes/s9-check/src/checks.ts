import { initSync, verify } from "./wasm/s9_wasm.js";
import { WASM_BASE64 } from "./wasm/s9_wasm_base64";
import { SAMPLE_MESSAGE, SAMPLE_SIGNATURE } from "./sample";

export type CheckResults = { sample: string; tampered: string; fetchBlocked: boolean };

function bytesFromBase64(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export async function runChecks(): Promise<CheckResults> {
  // The WASM is embedded in the script, so loading it needs no network request.
  initSync({ module: bytesFromBase64(WASM_BASE64) });
  const enc = new TextEncoder();
  const sample = verify(enc.encode(SAMPLE_MESSAGE), SAMPLE_SIGNATURE, "steplet-v1");
  const tampered = verify(enc.encode(SAMPLE_MESSAGE + "!"), SAMPLE_SIGNATURE, "steplet-v1");
  // Deliberately try to send something. Without the lock this would reach the preview server and get a 404 (no error).
  let fetchBlocked = false;
  try {
    await fetch("/s9-probe", { method: "POST", body: "spike" });
  } catch {
    fetchBlocked = true;
  }
  return { sample, tampered, fetchBlocked };
}
