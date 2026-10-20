<script lang="ts">
  // Screen 2: everything the file asks for, before anything runs (row 28). From top to bottom: the colour, who signed
  // it, the counts, what Steplet found, the file's own words in their marked panel, what is true on this computer,
  // and the Run button last, after the full list. The screen decides nothing: colour, findings and whether it can run
  // all come from Rust.
  import FileText from "../../shared/FileText.svelte";
  import Findings from "../../shared/Findings.svelte";
  import Icon from "../../shared/Icon.svelte";
  import Outcome, { type Colour } from "../../shared/Outcome.svelte";
  import type { EntryView } from "../../shared/types/EntryView";
  import type { PreviewView } from "../../shared/types/PreviewView";

  let { view, onrun }: { view: PreviewView; onrun: (typed: string | null) => void } = $props();

  /** The sentence a red file needs before Run works (J3 A, R9); Rust checks it again. */
  const SENTENCE = "I understand the risks and want to run this file";
  let typed = $state("");

  const colour: Colour = $derived(view.colour as Colour);
  const firstBlocked = $derived(view.findings.find((f) => f.severity === "blocked"));
  const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;
</script>

{#snippet entries(list: EntryView[], none: string, code = false, numbered = false)}
  {#if list.length === 0}
    <p class="none">{none}</p>
  {:else}
    <ol class="entries" class:numbered>
      {#each list as e, i (i)}
        <li class:code><span class="line-no">line {e.line}</span><FileText text={e.text} /></li>
      {/each}
    </ol>
  {/if}
{/snippet}

<article class="preview">
  <Outcome {colour} />

  <section class="identity" aria-label="Who signed this file">
    {#if view.key_changed}
      <p class="key-changed">
        <Icon kind="red" /><span
          >{view.identity.kind === "unsigned"
            ? "This sender name was used before on a signed file, and this file is not signed."
            : "This sender name was used before with another key."} See "On this computer", below.</span
        >
      </p>
    {/if}
    {#if view.identity.kind === "unchecked"}
      <p>Steplet could not check a signature, because it could not read the file.</p>
    {:else if view.identity.kind === "unsigned"}
      <p>Not signed. Steplet cannot tell who wrote this file.</p>
    {:else if view.identity.kind === "invalid"}
      <p>The signature does not match this file: it was changed after it was signed, or the signature is not real.</p>
    {:else if view.identity.kind === "new_key"}
      {#if view.key_changed}
        <p>The signature is valid, but this computer has not seen this key before.</p>
      {:else}
        <p>Signature valid, sender unknown: this computer has not seen this key before.</p>
      {/if}
      <p class="key">Key {view.identity.key}</p>
    {:else if view.identity.kind === "same_key"}
      <p>
        Signed with the same key as {plural(view.identity.runs, "file", "files")} from
        {#if view.file}<FileText text={view.file.sender.text} />{:else}this sender{/if} you ran here before.
      </p>
      <p class="key">Key {view.identity.key}</p>
    {:else if view.identity.kind === "key_known_as"}
      <p>
        Signed with a key this computer knows under another sender name:
        {#each view.identity.names as name, i (i)}{#if i > 0},
          {/if}<FileText text={name} />{/each}.
      </p>
      <p class="key">Key {view.identity.key}</p>
    {/if}
  </section>

  {#if view.file}
    <div class="numbers">
      <dl class="counts" aria-label="What it asks for, in numbers">
        <div><dt>Steps</dt><dd>{view.counts.steps}</dd></div>
        <div><dt>Commands</dt><dd>{view.counts.commands}</dd></div>
        <div><dt>Web sites</dt><dd>{view.counts.domains}</dd></div>
        <div><dt>Folders it reads</dt><dd>{view.counts.read}</dd></div>
        <div><dt>Folders it changes</dt><dd>{view.counts.write}</dd></div>
        <div><dt>Never rules</dt><dd>{view.counts.never}</dd></div>
      </dl>
      <p class="expires">Expires <span class="date">{view.file.expires}</span></p>
    </div>
  {/if}

  <section aria-labelledby="found-title">
    <h2 id="found-title">What Steplet found</h2>
    {#if view.findings.length === 0}
      <p class="none">Nothing to warn you about in this file.</p>
    {:else}
      <Findings findings={view.findings} />
    {/if}
  </section>

  {#if view.file}
    {@const f = view.file}
    <section aria-labelledby="file-title">
      <h2 id="file-title">From the file</h2>
      <p class="intro">Everything in this panel is the file's own text, shown exactly. None of it is Steplet's.</p>
      <div class="from-file" data-from-file-panel>
        <div class="row">
          <h3>Title</h3>
          <p class="title-text"><FileText text={f.title.text} /></p>
        </div>
        <div class="row">
          <h3>Sender</h3>
          <p><FileText text={f.sender.text} /></p>
        </div>
        <div class="row">
          <h3>Repository</h3>
          {#if f.repository}
            <p class="code"><FileText text={f.repository.url.text} /></p>
            <p class="sub">at commit <span class="code"><FileText text={f.repository.commit.text} /></span></p>
          {:else}
            <p class="none">None</p>
          {/if}
        </div>
        <div class="row">
          <h3>Steps</h3>
          {@render entries(f.steps, "None", false, true)}
        </div>
        <div class="row">
          <h3>Commands it may run, exactly as written</h3>
          {@render entries(f.commands, "None", true)}
        </div>
        <div class="row">
          <h3>Web sites it may fetch</h3>
          {@render entries(f.domains, "None", true)}
        </div>
        <div class="row">
          <h3>Folders it reads</h3>
          {@render entries(f.read, "None", true)}
        </div>
        <div class="row">
          <h3>Folders it changes</h3>
          {@render entries(f.write, "None", true)}
        </div>
        <div class="row">
          <h3>Never</h3>
          {@render entries(f.never, "None")}
        </div>
        <div class="row">
          <h3>The sender's notes</h3>
          {#if f.body.length === 0}
            <p class="none">None</p>
          {:else}
            <div class="body"><FileText text={f.body} block /></div>
          {/if}
        </div>
      </div>
    </section>
  {/if}

  {#if view.here.length > 0 || view.run_status.kind === "not_runnable"}
    <section aria-labelledby="here-title">
      <h2 id="here-title">On this computer</h2>
      {#if view.run_status.kind === "not_runnable"}
        <p class="here-line">{view.run_status.reason}</p>
      {/if}
      {#if view.here.length > 0}
        <Findings findings={view.here} />
      {/if}
    </section>
  {/if}

  <section class="run" aria-labelledby="run-title">
    {#if view.run_status.kind === "blocked"}
      <h2 id="run-title">This file cannot run</h2>
      {#if firstBlocked}<p>{firstBlocked.advice}</p>{/if}
    {:else if view.run_status.kind === "not_runnable"}
      <h2 id="run-title">Steplet cannot run this file on this computer</h2>
      <p>The reason is under "On this computer" above. You can still read everything the file asks for.</p>
    {:else}
      <h2 id="run-title">Run this file</h2>
      {#if view.run_status.kind === "red"}
        <label class="sentence" for="sentence">
          This file is red. To run it anyway, type this sentence exactly:
          <span class="say">{SENTENCE}</span>
        </label>
        <input
          id="sentence"
          class="field"
          type="text"
          autocomplete="off"
          spellcheck="false"
          bind:value={typed}
        />
      {/if}
      <div class="run-actions">
        <button
          class="button primary"
          type="button"
          disabled={view.run_status.kind === "red" && typed !== SENTENCE}
          onclick={() => onrun(view.run_status.kind === "red" ? typed : null)}>Run</button
        >
        <span class="muted">Next you choose how it runs. Nothing runs before that.</span>
      </div>
    {/if}
  </section>
</article>

<style>
  .preview {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
    padding-bottom: var(--space-12);
  }

  h2 {
    margin-bottom: var(--space-3);
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    line-height: 22px;
  }

  h3 {
    margin: 0 0 var(--space-1);
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 400;
    line-height: 18px;
  }

  .identity {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-top: calc(-1 * var(--space-4));
  }

  .key-changed {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    color: var(--red-fg);
    font-weight: 600;
  }

  /* The 16-pixel icon in the middle of the first 20-pixel line. */
  .key-changed :global(.icon) {
    flex: none;
    margin-top: 2px;
  }

  .key {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  .numbers {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  /* Six even boxes in one row; three in a row in a narrow window. */
  .counts {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: var(--space-2);
    margin: 0;
  }

  @media (max-width: 720px) {
    .counts {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }

  .counts div {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
  }

  .counts dt {
    color: var(--text-muted);
    font-size: 13px;
    line-height: 18px;
  }

  .counts dd {
    margin: var(--space-1) 0 0;
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 28px;
  }

  .expires {
    color: var(--text-muted);
  }

  .date {
    color: var(--text);
    font-weight: 600;
  }

  .none {
    color: var(--text-muted);
  }

  .intro {
    margin-bottom: var(--space-3);
    color: var(--text-muted);
  }

  .from-file {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-left: 4px solid var(--file-rule);
    border-radius: var(--radius-panel);
    background: var(--file-bg);
  }

  .title-text {
    font-family: var(--font-display);
    font-size: 18px;
    font-weight: 600;
    line-height: 26px;
  }

  .sub {
    margin-top: var(--space-1);
    color: var(--text-muted);
  }

  .code {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .entries {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .entries li {
    display: grid;
    grid-template-columns: 52px minmax(0, 1fr);
    gap: var(--space-2);
    align-items: baseline;
  }

  .entries li.code {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .line-no {
    color: var(--text-muted);
    font-family: var(--font-text);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .body {
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
  }

  .here-line {
    margin-bottom: var(--space-3);
  }

  .run {
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
  }

  .sentence {
    display: block;
    margin-bottom: var(--space-2);
  }

  .say {
    display: block;
    margin-top: var(--space-1);
    font-weight: 600;
  }

  .field {
    width: 100%;
    min-height: 32px;
    padding: 0 var(--space-3);
    border: 1px solid var(--control-border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }

  .run-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-4);
  }

  .muted {
    color: var(--text-muted);
  }
</style>
