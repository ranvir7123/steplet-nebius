<script lang="ts">
  // The app's one window. It shows one screen at a time; a file can be dropped on it anywhere. The backend is in the
  // context under "backend" (main.ts). The window's title stays "Steplet" whatever the file says: nothing here sets it.
  import { getContext, onMount } from "svelte";
  import { MAX_READ, type Backend } from "./lib/backend";
  import Open from "./screens/Open.svelte";
  import Preview from "./screens/Preview.svelte";
  import type { PreviewView } from "../shared/types/PreviewView";

  const backend = getContext<Backend>("backend");

  type Screen = { kind: "open" } | { kind: "preview"; view: PreviewView };
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

  function run(typed: string | null) {
    void typed; // The run chooser comes with Task 11.
  }
</script>

<svelte:window ondragover={dragover} ondrop={drop} />

<div class="window">
  <header class="bar">
    {#if screen.kind !== "open"}
      <button class="link" type="button" onclick={() => (screen = { kind: "open" })}>Open another file</button>
    {/if}
    <button class="link setup" type="button">Setup</button>
  </header>
  <main class="page">
    {#if screen.kind === "open"}
      <Open {reading} {error} {note} onfile={openFile} />
    {:else}
      <Preview view={screen.view} onrun={run} />
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
