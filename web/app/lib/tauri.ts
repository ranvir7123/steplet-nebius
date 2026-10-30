// The real backend: the app's own Tauri commands and events (plan section 6). Only the app build holds it.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ChooserView } from "../../shared/types/ChooserView";
import type { PreviewView } from "../../shared/types/PreviewView";
import type { ProgressView } from "../../shared/types/ProgressView";
import type { SetupView } from "../../shared/types/SetupView";
import type { Backend, Shown } from "./backend";

export const backend: Backend = {
  initialPreview: () => invoke<PreviewView | null>("initial_preview"),
  startNote: () => invoke<string | null>("start_note"),
  openBytes: (bytes) => invoke<PreviewView>("open_bytes", { bytes: Array.from(bytes) }),
  chooser: () => invoke<ChooserView>("chooser"),
  setupStatus: () => invoke<SetupView>("setup_status"),
  setupInstall: () => invoke<void>("setup_install"),
  setupRemove: () => invoke<SetupView>("setup_remove"),
  saveKey: (which, key) => invoke<SetupView>("save_key", { which, key }),
  forgetKey: (which) => invoke<SetupView>("forget_key", { which }),
  setOpensStepletFiles: (on) => invoke<SetupView>("set_opens_steplet_files", { on }),
  startRun: (sha256, typed, allowModelSwitch, runner) =>
    invoke<void>("start_run", { sha256, typed, allowModelSwitch, runner }),
  answer: (question, yes, fingerprint) => invoke<boolean>("answer", { question, yes, fingerprint }),
  stopRun: () => invoke<void>("stop_run"),
  stopAndClose: () => invoke<void>("stop_and_close"),
  saveReport: () => invoke<string | null>("save_report"),
  openHelp: (topic) => invoke<void>("open_help", { topic }),
  onRunProgress: (cb) => listen<ProgressView>("run-progress", (e) => cb(e.payload)),
  onSetupLine: (cb) => listen<Shown>("setup-line", (e) => cb(e.payload)),
  onCloseAsked: (cb) => listen("close-asked", () => cb()),
};
