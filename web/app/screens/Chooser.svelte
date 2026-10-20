<script lang="ts">
  // Screen 3: how the file runs. One runner can run it: Claude Code, in Steplet's own Linux, with the lines it may
  // promise for this file, each with the tests that back it (R6), and what leaves this computer (R16). Codex is listed
  // as not available, with its reason (D1). If Steplet's Linux or the key is not ready, the button leads to Setup.
  // The lines, the reason and whether it is ready all come from Rust.
  import { getContext, onMount } from "svelte";
  import FileText from "../../shared/FileText.svelte";
  import Icon from "../../shared/Icon.svelte";
  import type { ChooserView } from "../../shared/types/ChooserView";
  import type { Backend } from "../lib/backend";

  let { onback, onsetup, onrun }: { onback: () => void; onsetup: () => void; onrun: () => void } = $props();

  const backend = getContext<Backend>("backend");
  let view = $state<ChooserView | null>(null);
  let error = $state<string | null>(null);

  onMount(() => {
    backend.chooser().then(
      (v) => (view = v),
      (e) => (error = typeof e === "string" ? e : "Steplet could not get the run ready."),
    );
  });
</script>

<article class="chooser">
  <div class="heading">
    <h1>Choose how it runs</h1>
    <button class="link" type="button" onclick={onback}>Back to the file</button>
  </div>

  {#if error}
    <p class="message error" role="alert">{error}</p>
  {:else if !view}
    <p class="muted" role="status">Steplet is checking that everything is set up…</p>
  {:else}
    <section class="runner" aria-labelledby="claude-title">
      <h2 id="claude-title">Claude Code, in Steplet's own Linux</h2>

      <h3 id="promises-title">What Steplet makes sure of in this run</h3>
      <ul class="promises" aria-labelledby="promises-title">
        {#each view.claude.promises as p, i (i)}
          <li>
            <span class="tick"><Icon kind="green" /></span>
            <div class="promise">
              <p>{p.text}</p>
              {#if p.items.length > 0}
                <p class="sites">
                  {#each p.items as item, j (j)}<span class="site"><FileText text={item} /></span>{/each}
                </p>
              {/if}
              <details>
                <summary>The tests that show this</summary>
                <p class="tests">{#each p.tests as t (t)}<code>{t}</code>{/each}</p>
              </details>
            </div>
          </li>
        {/each}
      </ul>

      <h3>What leaves this computer</h3>
      <p>{view.claude.data_leaves}</p>

      <div class="actions">
        {#if view.claude.ready}
          <button class="button primary" type="button" onclick={onrun}>Run with Claude Code</button>
          <span class="muted">Steplet fetches the repository first, then the agent starts.</span>
        {:else}
          <button class="button primary" type="button" onclick={onsetup}>Set up first</button>
          <span class="muted">Steplet's own Linux or your key is not ready yet.</span>
        {/if}
      </div>
    </section>

    <section class="runner off" aria-labelledby="codex-title">
      <h2 id="codex-title">Codex</h2>
      <p>{view.codex_reason}</p>
    </section>
  {/if}
</article>

<style>
  .chooser {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding-bottom: var(--space-12);
  }

  .heading {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
  }

  h1 {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 28px;
  }

  h2 {
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    line-height: 22px;
  }

  h3 {
    margin: var(--space-5) 0 var(--space-2);
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 400;
    line-height: 18px;
  }

  .runner {
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
  }

  .off {
    background: var(--surface-sunken);
  }

  .off p {
    margin-top: var(--space-2);
    color: var(--text-muted);
  }

  .promises {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .promises li {
    display: grid;
    grid-template-columns: 16px minmax(0, 1fr);
    gap: var(--space-2);
  }

  .tick {
    padding-top: 2px;
    color: var(--green-fg);
  }

  .sites {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-2);
    margin-top: var(--space-2);
  }

  .site {
    padding: 0 var(--space-2);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 22px;
  }

  details {
    margin-top: var(--space-1);
    color: var(--text-muted);
    font-size: 13px;
  }

  summary {
    width: fit-content;
    cursor: pointer;
  }

  .tests {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-top: var(--space-1);
  }

  code {
    font-family: var(--font-mono);
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-6);
  }

  .muted {
    color: var(--text-muted);
  }

  .message {
    padding: var(--space-3) var(--space-4);
    border: 1px solid;
    border-radius: var(--radius-panel);
  }

  .error {
    border-color: var(--red-border);
    background: var(--red-bg);
    color: var(--red-fg);
  }
</style>
