<script lang="ts">
  // Screen 4: setup, once. A checklist of what a run needs (Windows' Linux feature, Steplet's own Linux, its tools,
  // its gate program), each with its state as a word and an icon; one button that installs Steplet's Linux, with what
  // it printed; the receiver's key; and the switch for opening .steplet files. Steplet never runs Windows' own
  // install for the receiver: it shows the command to type. Every line's words come from Rust; what setup's programs
  // print is file text.
  import { getContext, onMount } from "svelte";
  import FileText from "../../shared/FileText.svelte";
  import Icon from "../../shared/Icon.svelte";
  import type { LineView } from "../../shared/types/LineView";
  import type { SetupView } from "../../shared/types/SetupView";
  import type { Backend } from "../lib/backend";
  import { install as installLinux, remove as removeLinux, work } from "../lib/install.svelte";

  let { onback }: { onback: () => void } = $props();

  const backend = getContext<Backend>("backend");
  let view = $state<SetupView | null>(null);
  let looking = $state(true);
  let lookError = $state<string | null>(null);
  let askRemove = $state(false);
  let key = $state("");
  let keyError = $state<string | null>(null);
  let switchError = $state<string | null>(null);

  /** A refusal from Rust is Steplet's own sentence. */
  const sentence = (e: unknown) => (typeof e === "string" ? e : "Steplet could not do that.");

  async function check() {
    looking = true;
    lookError = null;
    try {
      view = await backend.setupStatus();
    } catch (e) {
      lookError = sentence(e);
    } finally {
      looking = false;
    }
  }

  onMount(() => void check());

  // An install or a removal keeps going if the receiver leaves this screen (lib/install.svelte.ts).
  async function install() {
    askRemove = false;
    await installLinux(backend);
    await check();
  }

  async function remove() {
    askRemove = false;
    const after = await removeLinux(backend);
    if (after) view = after;
  }

  async function saveKey() {
    keyError = null;
    const typed = key;
    key = "";
    try {
      view = await backend.saveKey(typed);
    } catch (e) {
      keyError = sentence(e);
    }
  }

  async function forgetKey() {
    keyError = null;
    try {
      view = await backend.forgetKey();
    } catch (e) {
      keyError = sentence(e);
    }
  }

  /** `box`: the checkbox, which the browser has already flipped; put back if Windows refused the change. */
  async function turn(on: boolean, box?: HTMLInputElement) {
    switchError = null;
    try {
      view = await backend.setOpensStepletFiles(on);
    } catch (e) {
      switchError = sentence(e);
      if (box && view) box.checked = view.opens_steplet_files !== "off";
    }
  }

  const word: Record<LineView["status"], string> = { ready: "Ready", missing: "Missing", unknown: "Not known yet" };
  const icon: Record<LineView["status"], "green" | "amber" | "note"> = {
    ready: "green",
    missing: "amber",
    unknown: "note",
  };
  const offered = $derived(view ? view.install !== "none" : false);
</script>

{#snippet item(l: LineView)}
  <li class="item">
    <span class="state {l.status}"><Icon kind={icon[l.status]} />{word[l.status]}</span>
    <div>
      <p class="item-text">{l.text}</p>
      {#if l.detail}<div class="printed"><FileText text={l.detail} block /></div>{/if}
    </div>
  </li>
{/snippet}

<article class="setup">
  <div class="heading">
    <h1>Set up Steplet</h1>
    <button class="link" type="button" onclick={onback}>Back</button>
  </div>
  <p class="intro">
    Steplet runs each file in a Linux of its own, inside Windows' Linux feature, so the agent never sees your own files.
    You set this up once.
  </p>
  <p class="promise">
    Steplet never asks for your password and never changes your own Windows or Linux, except what you switch on here.
  </p>

  {#if lookError}
    <p class="message error" role="alert">{lookError}</p>
  {/if}

  {#if !view}
    <p class="muted" role="status">Steplet is looking at what is set up…</p>
  {:else}
    <section class="panel" aria-labelledby="linux-title">
      <div class="panel-head">
        <h2 id="linux-title">Steplet's own Linux</h2>
        <button class="button" type="button" disabled={looking || work.running} onclick={check}>
          {looking ? "Looking…" : "Check again"}
        </button>
      </div>
      <ul class="checklist">
        {@render item(view.wsl)}
        {@render item(view.linux)}
        {@render item(view.tools)}
        {@render item(view.gate)}
      </ul>

      {#if view.wsl_fix}
        <div class="todo">
          <p>{view.wsl_fix.why}</p>
          <p><code class="command">{view.wsl_fix.command}</code></p>
          <p>{view.wsl_fix.then}</p>
          <button class="link" type="button" onclick={() => backend.openHelp("wsl")}>Microsoft's page about it</button>
        </div>
      {/if}

      {#if offered || work.running || work.failure || work.finished}
        <div class="install">
          {#if offered && !work.running && !work.failure}
            {#if view.install === "finish"}
              <p class="muted">
                This adds what is missing above to Steplet's own Linux. It is safe to repeat, can download a little, and
                takes a few minutes.
              </p>
            {:else}
              <p class="muted">
                This downloads Ubuntu 26.04 and Steplet's tools into Steplet's own Linux: about 1 to 2 GB, and a few
                minutes. WSL also adds a "steplet" entry to Windows Terminal and a "steplet" folder to the Start menu, as
                it does for every Linux.
              </p>
            {/if}
            <div class="actions">
              <button class="button primary" type="button" onclick={install}>
                {view.install === "finish" ? "Finish setting up Steplet's Linux" : "Install Steplet's Linux"}
              </button>
            </div>
          {/if}
          {#if work.running}
            <p class="muted" role="status">Steplet is changing its Linux. This can take several minutes…</p>
          {/if}
          {#if work.lines.length > 0}
            <div class="log" role="log" aria-label="What setup printed">
              {#each work.lines as l, i (i)}<FileText text={l} block />{/each}
            </div>
          {/if}
          {#if work.finished && !work.failure}
            <p class="done"><Icon kind="green" />Setup finished.</p>
          {/if}
          {#if work.failure}
            <div class="message error" role="alert">
              <p>{work.failure.reason}</p>
              {#if work.failure.detail}<div class="printed"><FileText text={work.failure.detail} block /></div>{/if}
            </div>
            <div class="actions">
              <button class="button primary" type="button" onclick={install}>Try again</button>
              <button class="button" type="button" onclick={() => (askRemove = true)}>Start over</button>
            </div>
          {/if}
          {#if askRemove}
            <div class="confirm">
              <p>
                Start over removes Steplet's own Linux and everything in it. Nothing of yours is in it. Then you can
                install it again.
              </p>
              <div class="actions">
                <button class="button primary" type="button" onclick={remove}>Remove Steplet's Linux</button>
                <button class="button" type="button" onclick={() => (askRemove = false)}>Keep it</button>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </section>

    <section class="panel" aria-labelledby="key-title">
      <h2 id="key-title">Your Anthropic key</h2>
      <p class="muted">
        Claude Code works with your own Anthropic API key. Steplet keeps it in Windows' Credential Manager, locked to your
        Windows account, and never shows it again.
      </p>
      {#if view.key_saved}
        <p class="saved"><Icon kind="green" />Saved</p>
      {/if}
      <label class="label" for="key">Anthropic API key</label>
      <div class="key-row">
        <input
          id="key"
          class="field"
          type="password"
          autocomplete="off"
          spellcheck="false"
          placeholder={view.key_saved ? "Paste a new key to replace the saved one" : "Paste your key"}
          bind:value={key}
        />
        <button class="button primary" type="button" disabled={key.trim() === ""} onclick={saveKey}>Save key</button>
        {#if view.key_saved}
          <button class="button" type="button" onclick={forgetKey}>Forget key</button>
        {/if}
      </div>
      {#if keyError}<p class="message error" role="alert">{keyError}</p>{/if}
      <button class="link" type="button" onclick={() => backend.openHelp("anthropic-key")}>Get a key from Anthropic</button>
    </section>

    <section class="panel" aria-labelledby="switch-title">
      <h2 id="switch-title">Opening .steplet files</h2>
      <label class="switch">
        <input
          type="checkbox"
          checked={view.opens_steplet_files !== "off"}
          onchange={(e) => turn(e.currentTarget.checked, e.currentTarget)}
        />
        Open .steplet files with Steplet
      </label>
      <p class="muted">
        Off until you turn it on. It is for your Windows user only, and turning it off removes Steplet's entries.
      </p>
      {#if view.opens_steplet_files === "broken"}
        <div class="message warn">
          <p>The switch is on, but it points at a Steplet program that is not this one: Steplet was moved or changed.</p>
          <button class="button" type="button" onclick={() => turn(true)}>Point it at this Steplet</button>
        </div>
      {/if}
      {#if switchError}<p class="message error" role="alert">{switchError}</p>{/if}
    </section>
  {/if}
</article>

<style>
  .setup {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
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
    margin-bottom: var(--space-3);
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    line-height: 22px;
  }

  .intro {
    margin-top: calc(-1 * var(--space-2));
  }

  .promise {
    padding: var(--space-3) var(--space-4);
    border-left: 3px solid var(--accent);
    background: var(--surface);
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
  }

  .panel h2 {
    margin-bottom: 0;
  }

  .checklist {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .item {
    display: grid;
    grid-template-columns: 132px minmax(0, 1fr);
    gap: var(--space-3);
    align-items: start;
  }

  @media (max-width: 560px) {
    .item {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--space-1);
    }
  }

  .state {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    width: fit-content;
    padding: 0 var(--space-2);
    border-radius: var(--radius);
    font-weight: 600;
    line-height: 24px;
  }

  .state.ready {
    background: var(--green-bg);
    color: var(--green-fg);
  }

  .state.missing {
    background: var(--amber-bg);
    color: var(--amber-fg);
  }

  .state.unknown {
    background: var(--surface-sunken);
    color: var(--text-muted);
  }

  .item-text {
    line-height: 24px;
  }

  .panel-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
  }

  .printed {
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .todo,
  .confirm {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-sunken);
  }

  .todo .link {
    align-self: flex-start;
  }

  .command {
    display: inline-block;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    font-family: var(--font-mono);
    font-size: 13px;
    user-select: all;
  }

  .install {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .log {
    max-height: 240px;
    overflow: auto;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 18px;
  }

  .done,
  .saved {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--green-fg);
    font-weight: 600;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
  }

  .label {
    font-weight: 600;
  }

  .key-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .field {
    flex: 1 1 260px;
    min-height: 32px;
    padding: 0 var(--space-3);
    border: 1px solid var(--control-border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }

  .panel > .link {
    align-self: flex-start;
  }

  .switch {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 600;
    cursor: pointer;
  }

  .switch input {
    width: 16px;
    height: 16px;
    margin: 0;
    accent-color: var(--accent);
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

  .error .printed {
    color: var(--text);
  }

  .warn {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    align-items: flex-start;
    border-color: var(--amber-border);
    background: var(--amber-bg);
    color: var(--amber-fg);
  }
</style>
