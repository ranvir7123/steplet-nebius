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
  openPath: (path) => invoke<PreviewView>("open_path", { path }),
  openBytes: (bytes) => invoke<PreviewView>("open_bytes", { bytes: Array.from(bytes) }),
  chooser: () => invoke<ChooserView>("chooser"),
  setupStatus: () => invoke<SetupView>("setup_status"),
  setupInstall: () => invoke<void>("setup_install"),
  saveKey: (key) => invoke<SetupView>("save_key", { key }),
  forgetKey: () => invoke<SetupView>("forget_key"),
  setOpensStepletFiles: (on) => invoke<SetupView>("set_opens_steplet_files", { on }),
  startRun: (sha256, typed) => invoke<void>("start_run", { sha256, typed }),
  answer: (question, yes, fingerprint) => invoke<boolean>("answer", { question, yes, fingerprint }),
  stopRun: () => invoke<void>("stop_run"),
  saveReport: () => invoke<string | null>("save_report"),
  openHelp: (topic) => invoke<void>("open_help", { topic }),
  onRunProgress: (cb) => listen<ProgressView>("run-progress", (e) => cb(e.payload)),
  onSetupLine: (cb) => listen<Shown>("setup-line", (e) => cb(e.payload)),
};
