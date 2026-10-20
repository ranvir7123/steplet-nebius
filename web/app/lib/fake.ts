// The stand-in backend, for screen tests only (only the "screens" build holds it, see vite.config.ts). It replays a
// scenario saved from real Rust output, chosen by the page's address: ?scenario=<name>, one of the files in
// web/app/tests/fixtures. It decides nothing itself: every answer, word and colour comes from the saved data.
import type { Fixture } from "../../shared/types/Fixture";
import type { ProgressView } from "../../shared/types/ProgressView";
import type { SetupChanges } from "../../shared/types/SetupChanges";
import type { SetupView } from "../../shared/types/SetupView";
import type { Backend, Shown, Unlisten } from "./backend";
import { timing } from "./scenarios";

/** A mark only this file holds; a test checks that the app build does not contain it. */
export const STAND_IN_MARK = "steplet-stand-in-backend";

const saved = import.meta.glob<Fixture>("../tests/fixtures/*.json", { import: "default" });

const name = new URLSearchParams(location.search).get("scenario") ?? "open-empty";
const t = timing(name);

let loading: Promise<Fixture> | null = null;
function fixture(): Promise<Fixture> {
  if (!loading) {
    const load = saved[`../tests/fixtures/${name}.json`];
    loading = load ? load() : Promise.reject(new Error(`${STAND_IN_MARK}: no saved scenario named ${name}`));
  }
  return loading;
}

const wait = (ms: number) => new Promise<void>((done) => setTimeout(done, ms));
const never = () => new Promise<never>(() => {});

type Listener<T> = (value: T) => void;
const progressListeners = new Set<Listener<ProgressView>>();
const lineListeners = new Set<Listener<Shown>>();

function listenTo<T>(set: Set<Listener<T>>, cb: Listener<T>): Promise<Unlisten> {
  set.add(cb);
  return Promise.resolve(() => set.delete(cb));
}

async function emit(items: ProgressView[]) {
  for (const item of items) {
    await wait(t.step);
    for (const cb of progressListeners) cb(item);
  }
}

let setup: SetupView | null = null;
async function currentSetup(): Promise<SetupView> {
  if (!setup) {
    const f = await fixture();
    if (!f.setup) throw new Error(`${STAND_IN_MARK}: scenario ${name} has no setup`);
    setup = f.setup;
  }
  return setup;
}

/** The setup screen after each change, as the Rust side worked it out when the scenario was saved. */
async function changes(): Promise<SetupChanges> {
  const f = await fixture();
  if (!f.setup_changes) throw new Error(`${STAND_IN_MARK}: scenario ${name} has no setup changes`);
  return f.setup_changes;
}

async function preview() {
  const f = await fixture();
  if (t.open === null) return never();
  await wait(t.open);
  if (f.open_error) throw new Error(f.open_error);
  if (!f.preview) throw new Error(`${STAND_IN_MARK}: scenario ${name} has no preview`);
  return f.preview;
}

export const backend: Backend = {
  async initialPreview() {
    const f = await fixture();
    return f.screen === "open" ? null : f.preview;
  },
  async startNote() {
    return (await fixture()).note;
  },
  openPath: () => preview(),
  openBytes: () => preview(),
  async chooser() {
    const f = await fixture();
    if (!f.chooser) throw new Error(`${STAND_IN_MARK}: scenario ${name} has no chooser`);
    return f.chooser;
  },
  setupStatus: () => currentSetup(),
  async setupInstall() {
    const f = await fixture();
    for (const l of f.setup_lines) {
      await wait(t.line);
      for (const cb of lineListeners) cb(l);
    }
    if (!t.installEnds) return never();
    if (f.setup_error) throw f.setup_error;
  },
  async saveKey() {
    setup = (await changes()).key_saved;
    return setup;
  },
  async forgetKey() {
    setup = (await changes()).key_forgotten;
    return setup;
  },
  async setOpensStepletFiles(on) {
    const c = await changes();
    setup = on ? c.switch_on : c.switch_off;
    return setup;
  },
  async startRun() {
    void emit((await fixture()).progress);
  },
  async answer(question, yes) {
    const f = await fixture();
    void emit([{ kind: "question_closed", id: question, how: yes ? f.replies.yes : f.replies.no }]);
    return true;
  },
  async stopRun() {
    void emit((await fixture()).after_stop);
  },
  async saveReport() {
    const f = await fixture();
    if (f.save_error) throw new Error(f.save_error);
    return f.saved_to;
  },
  async openHelp() {},
  onRunProgress: (cb) => listenTo(progressListeners, cb),
  onSetupLine: (cb) => listenTo(lineListeners, cb),
};
