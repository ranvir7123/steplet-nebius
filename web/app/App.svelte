<script lang="ts">
  // The app's one window. It shows one screen at a time; a file can be dropped on it anywhere. The backend is in the
  // context under "backend" (main.ts). The window's title stays "Steplet" whatever the file says: nothing here sets it.
  import { getContext, onMount } from "svelte";
  import { MAX_READ, type Backend } from "./lib/backend";
  import Chooser from "./screens/Chooser.svelte";
  import Open from "./screens/Open.svelte";
  import Preview from "./screens/Preview.svelte";
  import Setup from "./screens/Setup.svelte";
  import type { PreviewView } from "../shared/types/PreviewView";

  const backend = getContext<Backend>("backend");

  type Screen =
    | { kind: "open" }
    | { kind: "preview"; view: PreviewView }
    // `typed`: the red file's sentence, for Rust to check again when the run starts.
    | { kind: "chooser"; view: PreviewView; typed: string | null }
    // `back`: the screen Setup was opened from.
    | { kind: "setup"; back: Screen };
  let screen = $state<Screen>({ kind: "open" });
  let reading = $state(false);
  let error = $state<string | null>(null);
  let note = $state<string | null>(null);

  /** A refusal from Rust is Steplet's own sentence; anything else gets one. */
  function words(e: unknown): string {
    if (typeof e === "string") return e;
    return "Steplet could not open this file.";
  }

  onMount(() => {
    backend.startNote().then(
      (n) => (note = n),
      () => {},
    );
    backend.initialPreview().then(
      (view) => {
        if (view) screen = { kind: "preview", view };
      },
      (e) => (error = words(e)),
    );
  });

  async function openFile(file: File) {
    error = null;
    reading = true;
    try {
      const bytes = new Uint8Array(await file.slice(0, MAX_READ).arrayBuffer());
      screen = { kind: "preview", view: await backend.openBytes(bytes) };
      window.scrollTo(0, 0);
    } catch (e) {
      error = words(e);
      screen = { kind: "open" };
    } finally {
      reading = false;
    }
  }

  function dragover(e: DragEvent) {
    e.preventDefault();
  }

  function drop(e: DragEvent) {
    e.preventDefault();
    const file = e.dataTransfer?.files[0];
    if (file) void openFile(file);
  }

  function choose(view: PreviewView, typed: string | null) {
    screen = { kind: "chooser", view, typed };
    window.scrollTo(0, 0);
  }

  function setup() {
    if (screen.kind !== "setup") screen = { kind: "setup", back: screen };
    window.scrollTo(0, 0);
  }

  function back(to: Screen) {
    screen = to;
    window.scrollTo(0, 0);
  }

  function start() {
    // The run itself comes with Task 12 (prepare, the live log and the ask screen).
  }
</script>

<svelte:window ondragover={dragover} ondrop={drop} />

<div class="window">
  <header class="bar">
    {#if screen.kind === "preview" || screen.kind === "chooser"}
      <button class="link" type="button" onclick={() => back({ kind: "open" })}>Open another file</button>
    {/if}
    {#if screen.kind !== "setup"}
      <button class="link setup" type="button" onclick={setup}>Setup</button>
    {/if}
  </header>
  <main class="page">
    {#if screen.kind === "open"}
      <Open {reading} {error} {note} onfile={openFile} />
    {:else if screen.kind === "preview"}
      {@const view = screen.view}
      <Preview {view} onrun={(typed) => choose(view, typed)} />
    {:else if screen.kind === "chooser"}
      {@const view = screen.view}
      <Chooser onback={() => back({ kind: "preview", view })} onsetup={setup} onrun={start} />
    {:else}
      {@const to = screen.back}
      <Setup onback={() => back(to)} />
    {/if}
  </main>
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .bar {
    display: flex;
    align-items: center;
    width: 100%;
    max-width: calc(var(--column) + 2 * var(--space-6));
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) 0;
  }

  .setup {
    margin-left: auto;
  }

  .page {
    display: flex;
    flex: 1;
    flex-direction: column;
    width: 100%;
    max-width: calc(var(--column) + 2 * var(--space-6));
    margin: 0 auto;
    padding: var(--space-6);
  }
</style>
