<script lang="ts">
  // Screen 3: who runs the file. Steplet's own agent comes first, on a Nemotron model at Nebius Token Factory, with
  // its price, where it stops and the lines it may promise for this file; Claude Code, in Steplet's own Linux, comes
  // second with its own lines. Each line names the tests that back it (R6), and each card says what leaves this
  // computer (R16). Codex is listed as not available, with its reason (D1). A card whose runner is not ready says what
  // it needs, and its button leads to Setup. The lines, the models, the reason and what each needs come from Rust.
  import { getContext, onMount } from "svelte";
  import FileText from "../../shared/FileText.svelte";
  import Icon from "../../shared/Icon.svelte";
  import type { ChooserView } from "../../shared/types/ChooserView";
  import type { Runner } from "../../shared/types/Runner";
  import type { Backend } from "../lib/backend";

  let { onback, onsetup, onrun }: { onback: () => void; onsetup: () => void; onrun: (runner: Runner) => void } =
    $props();

  const backend = getContext<Backend>("backend");
  let view = $state<ChooserView | null>(null);
  let error = $state<string | null>(null);
  /** The model Steplet's agent runs on: the one Rust chose at first, until the receiver picks another. */
  let model = $state<string | null>(null);

  onMount(() => {
    backend.chooser().then(
      (v) => {
        view = v;
        model = v.runners.find((r) => r.kind === "own")?.model ?? null;
      },
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
    {#each view.runners as r (r.kind)}
      {@const own = r.kind === "own"}
      {@const chosen = r.models.find((m) => m.id === model)}
      <section class="runner" aria-labelledby="{r.kind}-title">
        <h2 id="{r.kind}-title">{r.name}</h2>
        {#if own && chosen}
          <p class="subtitle">{chosen.name}, on Nebius Token Factory</p>
        {/if}

        {#if r.models.length > 0}
          <h3 id="{r.kind}-models">Model</h3>
          {#if r.models.length > 1}
            <div class="models" role="radiogroup" aria-labelledby="{r.kind}-models">
              {#each r.models as m (m.id)}
                <label class="model choice">
                  <input type="radio" name="{r.kind}-model" value={m.id} bind:group={model} />
                  <span class="about">
                    <span class="name">{m.name}</span>
                    <span class="price">{m.price}</span>
                    <code class="id">{m.id}</code>
                  </span>
                </label>
              {/each}
            </div>
          {:else}
            {@const m = r.models[0]}
            <div class="models">
              <div class="model">
                <span class="about">
                  <span class="name">{m.name}</span>
                  <span class="price">{m.price}</span>
                  <code class="id">{m.id}</code>
                </span>
              </div>
            </div>
          {/if}
        {/if}

        <h3 id="{r.kind}-promises">What Steplet makes sure of in this run</h3>
        <ul class="promises" aria-labelledby="{r.kind}-promises">
          {#each r.limits ? [r.limits, ...r.promises] : r.promises as p, i (i)}
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
        <p>{r.data_leaves}</p>

        <!-- Steplet's agent is the main way to run; Claude Code is the other. -->
        <div class="actions">
          {#if r.needs}
            <button class="button" class:primary={own} type="button" onclick={onsetup}>Set up first</button>
            <span class="muted">{r.needs}</span>
          {:else if own && chosen}
            <button class="button primary" type="button" onclick={() => onrun({ kind: "own", model: chosen.id })}>
              Run with Steplet's agent
            </button>
            <span class="muted">Steplet fetches the repository first, then the agent starts.</span>
          {:else if !own}
            <button class="button" type="button" onclick={() => onrun({ kind: "claude_code" })}>
              Run with Claude Code
            </button>
            <span class="muted">Steplet fetches the repository first, then the agent starts.</span>
          {/if}
        </div>
      </section>
    {/each}

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

  .subtitle {
    margin-top: var(--space-1);
    color: var(--text-muted);
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

  .models {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .model {
    display: grid;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .choice {
    grid-template-columns: 16px minmax(0, 1fr);
    gap: var(--space-2);
    cursor: pointer;
  }

  .choice:hover {
    border-color: var(--control-border);
  }

  .choice:has(input:checked) {
    border-color: var(--accent);
  }

  input[type="radio"] {
    width: 16px;
    height: 16px;
    margin: 2px 0 0;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .about {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-weight: 600;
  }

  .price {
    color: var(--text-muted);
    font-size: 13px;
    line-height: 18px;
  }

  .id {
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
