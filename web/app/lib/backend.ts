// The one way the screens talk to Steplet's Rust side (plan section 6). Two versions satisfy it: the real one
// (tauri.ts, the app's own commands and events) and a stand-in for screen tests (fake.ts, which replays data saved
// from real Rust output). The screens import `backend` from "$backend"; the build picks the version (vite.config.ts),
// so the app never holds the stand-in.
import type { ChooserView } from "../../shared/types/ChooserView";
import type { PreviewView } from "../../shared/types/PreviewView";
import type { ProgressView } from "../../shared/types/ProgressView";
import type { Seg } from "../../shared/types/Seg";
import type { SetupView } from "../../shared/types/SetupView";

/** File, repository, agent or command text, cut into display pieces; drawn only by shared/FileText.svelte. */
export type Shown = Seg[];

/** Stops listening for events. */
export type Unlisten = () => void;

/** The most bytes of a file the page sends: one more than Steplet takes (crates/steplet-app/src/open.rs). */
export const MAX_READ = 256 * 1024 + 1;

export interface Backend {
  /** The file Steplet was opened with (a double-click), if any. */
  initialPreview(): Promise<PreviewView | null>;
  /** A note for the Open screen, in Steplet's words: a run left by a crash was cleared. */
  startNote(): Promise<string | null>;
  /** A file dropped on the window or chosen with "Open a file…": at most its first MAX_READ bytes. A page cannot
   * learn a file's path, so it never asks Rust to read one. */
  openBytes(bytes: Uint8Array): Promise<PreviewView>;
  chooser(): Promise<ChooserView>;
  setupStatus(): Promise<SetupView>;
  /** Installs Steplet's Linux; each line it prints comes through `onSetupLine`. A failure rejects with a FailureView. */
  setupInstall(): Promise<void>;
  saveKey(key: string): Promise<SetupView>;
  forgetKey(): Promise<SetupView>;
  setOpensStepletFiles(on: boolean): Promise<SetupView>;
  /** Starts the run of the file previewed with this SHA-256; `typed` is the red file's sentence. */
  startRun(sha256: string, typed: string | null): Promise<void>;
  /** The receiver's answer to a question, bound to the request's fingerprint; false when no such question is open. */
  answer(question: number, yes: boolean, fingerprint: string): Promise<boolean>;
  stopRun(): Promise<void>;
  /** Saves the report through Windows' Save dialog; where it was saved, or null when the receiver cancelled. */
  saveReport(): Promise<string | null>;
  /** Opens one of Steplet's own help pages in the browser, from a fixed list (S11). */
  openHelp(topic: string): Promise<void>;
  onRunProgress(cb: (p: ProgressView) => void): Promise<Unlisten>;
  onSetupLine(cb: (line: Shown) => void): Promise<Unlisten>;
}
