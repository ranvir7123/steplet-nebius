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
  /** Whether the first answer to a question fails on its way to Rust, as a broken channel to the window would. */
  answerFails: boolean;
  /** Milliseconds a run takes to stop after Stop, before what it did after the stop arrives. */
  stopping: number;
}

const usual: Timing = { open: 0, step: 0, line: 0, installEnds: true, answerFails: false, stopping: 0 };

const special: Record<string, Partial<Timing>> = {
  "open-reading": { open: null },
  "setup-installing": { installEnds: false },
  // A question that is on screen a moment before the run takes it back.
  "ask-withdrawn": { step: 300 },
  "ask-answer-fails": { answerFails: true },
  // Rows that keep coming for a few seconds, so the receiver can scroll up while they do.
  "run-40-commands": { step: 40 },
  // A run that takes a moment to stop, as a real one does, so "Stopping…" can be seen.
  "run-stopping": { stopping: 1500 },
};

/** The timing of the scenario `name`. */
export function timing(name: string): Timing {
  return { ...usual, ...special[name] };
}
