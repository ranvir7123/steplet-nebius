// How the stand-in backend plays each saved scenario over time. The data comes from web/app/tests/fixtures (written
// by crates/steplet-app/tests/fixtures.rs); only the timing is here. A run whose saved progress has no end stays
// open (fetching, running, waiting on a question) until Stop.

export interface Timing {
  /** Milliseconds before openBytes answers; `null`: it never answers (the "reading" state). */
  open: number | null;
  /** Milliseconds between two progress items of a run, and before the first. */
  step: number;
  /** Milliseconds between two setup lines. */
  line: number;
  /** Whether the install ends after its last line (with the saved error, if there is one). */
  installEnds: boolean;
}

const usual: Timing = { open: 0, step: 0, line: 0, installEnds: true };

const special: Record<string, Partial<Timing>> = {
  "open-reading": { open: null },
  "setup-installing": { installEnds: false },
};

/** The timing of the scenario `name`. */
export function timing(name: string): Timing {
  return { ...usual, ...special[name] };
}
