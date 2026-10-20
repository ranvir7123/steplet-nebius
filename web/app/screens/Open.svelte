<script lang="ts">
  // Screen 1: open a file. Nothing in it runs until Run is pressed. A file can also be dropped anywhere on the window
  // (App.svelte); "Open a file…" uses Windows' own Open dialog, through the page's file picker.
  let {
    reading,
    error,
    note,
    onfile,
  }: { reading: boolean; error: string | null; note: string | null; onfile: (file: File) => void } = $props();

  let picker = $state<HTMLInputElement>();

  function chosen() {
    if (!picker) return;
    const file = picker.files?.[0];
    picker.value = "";
    if (file) onfile(file);
  }
</script>

<div class="open-screen">
  {#if note}
    <p class="message note" role="status">{note}</p>
  {/if}
  {#if error}
    <p class="message error" role="alert">{error}</p>
  {/if}
  <section class="open" aria-labelledby="open-title">
    <svg class="mark" viewBox="0 0 48 48" width="48" height="48" aria-hidden="true">
      <rect class="mark-square" width="48" height="48" rx="10.5" />
      <path class="mark-steps" d="M10.9 36.4V27.6h8.7v-8.7h8.7v-8.8h8.8v26.3z" />
    </svg>
    <h1 id="open-title">Open a .steplet file to see what it asks for.</h1>
    <p class="muted">Nothing in it runs until you press Run.</p>
    <div class="actions">
      <button class="button primary" type="button" disabled={reading} onclick={() => picker?.click()}>Open a file…</button>
      <span class="muted">or drop it on this window</span>
    </div>
    <p class="reading" role="status" aria-live="polite">{reading ? "Reading the file…" : ""}</p>
    <input
      class="picker"
      type="file"
      accept=".steplet"
      tabindex="-1"
      aria-hidden="true"
      bind:this={picker}
      onchange={chosen}
    />
  </section>
</div>

<style>
  .open-screen {
    display: flex;
    flex: 1;
    flex-direction: column;
    justify-content: center;
    gap: var(--space-4);
  }

  .message {
    padding: var(--space-3) var(--space-4);
    border: 1px solid;
    border-radius: var(--radius-panel);
  }

  .note {
    border-color: var(--border);
    background: var(--surface);
  }

  .error {
    border-color: var(--red-border);
    background: var(--red-bg);
    color: var(--red-fg);
  }

  .open {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-12) var(--space-6) var(--space-8);
    border: 1px dashed var(--control-border);
    border-radius: var(--radius-panel);
    background: var(--surface);
    text-align: center;
  }

  .mark {
    margin-bottom: var(--space-2);
  }

  .mark-square {
    fill: var(--accent);
  }

  .mark-steps {
    fill: var(--on-accent);
  }

  h1 {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 28px;
  }

  .muted {
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    margin-top: var(--space-4);
  }

  .reading {
    min-height: 20px;
    color: var(--text-muted);
  }

  .picker {
    display: none;
  }
</style>
