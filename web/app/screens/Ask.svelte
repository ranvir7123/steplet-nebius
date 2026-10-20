<script lang="ts">
  // Screen 7: the agent wants something the file did not ask for, and the run waits on the receiver's answer (R7).
  // The question covers the window: what exactly, with the whole request and nothing cut; why Steplet asks; what the
  // agent said just before; what still holds on a yes; and how long until no answer means no. "No" has the focus;
  // "Yes" works only once the question has been on screen for a second, so a click meant for something else cannot
  // answer it. Every word about the request comes from Rust; the request itself is file text.
  import { onDestroy } from "svelte";
  import FileText from "../../shared/FileText.svelte";
  import type { QuestionView } from "../../shared/types/QuestionView";
  import type { Shown } from "../lib/backend";
  import { modal } from "../lib/modal";
  import { same } from "../lib/shown";

  let {
    question,
    waiting,
    said,
    onanswer,
  }: {
    question: QuestionView;
    /** How many more questions wait behind this one. */
    waiting: number;
    /** What the agent said last before it asked, if anything. */
    said: Shown | null;
    /** Rejects when the answer did not get through to Steplet. */
    onanswer: (yes: boolean) => Promise<boolean>;
  } = $props();

  const HOLD_MS = 1000;
  // The time is counted on this page from when the view came: Rust says how long the question had waited by then.
  const came = performance.now();
  const left = () => Math.max(0, question.limit_ms - question.waited_ms - (performance.now() - came));
  let remaining = $state(left());
  let ready = $state(false);
  let answered = $state(false);
  let failed = $state(false);
  const hold = setTimeout(() => (ready = true), HOLD_MS);
  const tick = setInterval(() => (remaining = left()), 250);
  onDestroy(() => {
    clearTimeout(hold);
    clearInterval(tick);
  });

  const clock = (ms: number) => {
    const s = Math.ceil(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  };

  // The field that holds what is asked about comes first (the command, the file, the address); then the others.
  const fields = $derived([
    ...question.fields.filter((f) => same(f.value, question.what)),
    ...question.fields.filter((f) => !same(f.value, question.what)),
  ]);

  async function answer(yes: boolean) {
    if (answered || (yes && !ready)) return;
    answered = true;
    failed = false;
    try {
      // When the question is no longer open, its close is on its way and the buttons stay off.
      await onanswer(yes);
    } catch {
      answered = false;
      failed = true;
    }
  }
</script>

<dialog class="ask" aria-labelledby="ask-title" aria-describedby="ask-why" use:modal>
  <header class="head">
    <p class="kicker">The agent asks for something the file did not ask for</p>
    <h2 id="ask-title">{question.ask}</h2>
  </header>

  <div class="body">
    <dl class="request" aria-label="The whole request">
      {#each fields as f, i (i)}
        <div class="field">
          <dt class="name"><FileText text={f.name} /></dt>
          <dd class="value"><FileText text={f.value} block /></dd>
        </div>
      {/each}
    </dl>

    <p id="ask-why"><span class="label">Why Steplet asks:</span> {question.reason}.</p>

    {#if said}
      <div class="said">
        <p class="said-label">The agent said just before</p>
        <FileText text={said} block />
      </div>
    {/if}

    <p class="holds">{question.holds}</p>
  </div>

  <footer class="foot">
    <p class="countdown" role="timer" aria-live="off">No answer in {clock(remaining)} means no.</p>
    {#if waiting > 0}
      <p class="waiting">{waiting === 1 ? "1 more question is waiting." : `${waiting} more questions are waiting.`}</p>
    {/if}
    {#if failed}
      <p class="failed" role="alert">Steplet could not pass on your answer. Try again.</p>
    {/if}
    <div class="actions">
      <button class="button primary" type="button" data-first disabled={answered} onclick={() => answer(false)}>No</button>
      <button class="button" type="button" disabled={!ready || answered} onclick={() => answer(true)}>
        Yes, allow this exact request for this run
      </button>
    </div>
  </footer>
</dialog>

<style>
  .ask {
    display: flex;
    flex-direction: column;
    width: min(680px, calc(100vw - 2 * var(--space-4)));
    max-height: calc(100vh - 2 * var(--space-6));
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 16px 48px rgb(0 0 0 / 28%);
  }

  .ask::backdrop {
    background: rgb(0 0 0 / 45%);
  }

  .head {
    padding: var(--space-5) var(--space-6) var(--space-3);
    border-bottom: 1px solid var(--border);
  }

  .kicker {
    color: var(--text-muted);
  }

  h2 {
    margin-top: var(--space-1);
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 28px;
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-height: 0;
    overflow: auto;
    padding: var(--space-4) var(--space-6);
  }

  .request {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .name {
    color: var(--file-label);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 16px;
  }

  .value {
    max-height: 240px;
    overflow: auto;
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 20px;
  }

  .label {
    font-weight: 600;
  }

  .said {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
  }

  .said-label {
    color: var(--file-label);
    font-size: 12px;
    font-weight: 600;
    line-height: 16px;
  }

  .holds {
    padding: var(--space-3) var(--space-4);
    border-left: 3px solid var(--accent);
    background: var(--surface-sunken);
  }

  .foot {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-6) var(--space-5);
    border-top: 1px solid var(--border);
  }

  .countdown {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .waiting {
    color: var(--text-muted);
  }

  .failed {
    color: var(--red-fg);
    font-weight: 600;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-top: var(--space-1);
  }

  .actions .primary {
    min-width: 96px;
  }
</style>
