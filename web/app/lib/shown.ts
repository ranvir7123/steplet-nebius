// Small helpers for display pieces (file text) that only cut or measure them: they never change a character.
import type { Shown } from "./backend";

/** The first `n` lines of `text`, whole pieces kept as they are, and the piece the cut falls in cut after its line
 * break. The log shows this much of a command's output until the receiver asks for all of it. */
export function firstLines(text: Shown, n: number): Shown {
  const out: Shown = [];
  let left = n;
  for (const s of text) {
    if (s.kind !== "text") {
      out.push(s);
      continue;
    }
    let at = 0;
    while (left > 0) {
      const next = s.text.indexOf("\n", at);
      if (next === -1) break;
      at = next + 1;
      left -= 1;
    }
    if (left === 0) {
      if (at > 0) out.push({ kind: "text", text: s.text.slice(0, at) });
      return out;
    }
    out.push(s);
  }
  return out;
}

/** Whether two pieces of display text are the same text. */
export function same(a: Shown, b: Shown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
