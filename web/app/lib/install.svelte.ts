// An install or a removal of Steplet's Linux outlives the Setup screen: the receiver may go Back, or drop a file on
// the window, and come back while it runs. So what it printed, and how it ended, live here, for as long as the window.
import type { FailureView } from "../../shared/types/FailureView";
import type { SetupView } from "../../shared/types/SetupView";
import type { Backend, Shown } from "./backend";

export const work = $state({
  /** An install or a removal is going on. */
  running: false,
  /** What setup printed (file text), since the last install started. */
  lines: [] as Shown[],
  /** How the last one failed, if it did. */
  failure: null as FailureView | null,
  /** The last install ended without a failure. */
  finished: false,
});

let listening: Promise<unknown> | null = null;

/** Lines are kept from the first install on, whichever screen is showing. */
function listen(backend: Backend): Promise<unknown> {
  listening ??= backend.onSetupLine((l) => work.lines.push(l));
  return listening;
}

/** A refusal from Rust is Steplet's own words: a sentence, or a sentence with what a program printed. */
function failed(e: unknown): FailureView {
  if (typeof e === "object" && e !== null && "reason" in e) return e as FailureView;
  return { reason: typeof e === "string" ? e : "Steplet could not finish this.", detail: null };
}

/** Installs Steplet's Linux, or finishes setting it up. Does nothing while another install or removal runs. */
export async function install(backend: Backend): Promise<void> {
  if (work.running) return;
  work.running = true;
  work.lines = [];
  work.failure = null;
  work.finished = false;
  try {
    await listen(backend);
    await backend.setupInstall();
    work.finished = true;
  } catch (e) {
    work.failure = failed(e);
  } finally {
    work.running = false;
  }
}

/** "Start over": removes Steplet's Linux. The Setup screen as it is after, or `null` if it failed. */
export async function remove(backend: Backend): Promise<SetupView | null> {
  if (work.running) return null;
  work.running = true;
  work.lines = [];
  work.failure = null;
  work.finished = false;
  try {
    return await backend.setupRemove();
  } catch (e) {
    work.failure = failed(e);
    return null;
  } finally {
    work.running = false;
  }
}
