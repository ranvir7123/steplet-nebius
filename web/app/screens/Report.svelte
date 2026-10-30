<script lang="ts">
  // Screen 8: the report. First how the run ended, in Steplet's words, with what to do next; then every command with
  // its check, its exit code (from Claude Code's own record, never from the output) and everything it printed; what
  // Steplet refused; your answers; every model used and any refusal, and for Steplet's own agent what the run spent;
  // what changed in the working copy (already deleted); the trust check (Claude Code's runs only); what went wrong
  // around the run; Steplet's notes; and last, labelled, what the agent says. Every word about the run comes from
  // Rust; everything the agent, a command or a model wrote is file text.
  import { getContext } from "svelte";
  import FileText from "../../shared/FileText.svelte";
  import Icon from "../../shared/Icon.svelte";
  import type { CheckView } from "../../shared/types/CheckView";
  import type { ReportView } from "../../shared/types/ReportView";
  import type { Backend } from "../lib/backend";
  import { outcomeIcon } from "../lib/end";

  let {
    report,
    onrun,
    onopen,
  }: {
    report: ReportView;
    /** Run the file again; `allowModelSwitch` after a model refused (E1). */
    onrun: (allowModelSwitch: boolean) => void;
    onopen: () => void;
  } = $props();

  const backend = getContext<Backend>("backend");
  const end = $derived({ ...report.end_words, icon: outcomeIcon(report.end_words.icon) });
  let saving = $state(false);
  let savedAs = $state<string | null>(null);
  let saveError = $state<string | null>(null);

  async function save() {
    saving = true;
    savedAs = null;
    saveError = null;
    try {
      savedAs = await backend.saveReport();
    } catch (e) {
      saveError = typeof e === "string" ? e : "Steplet could not save the report.";
    } finally {
      saving = false;
    }
  }

  const checkIcon = (c: CheckView): "green" | "amber" | "red" | "note" =>
    c.kind === "allowed" || c.kind === "allowed_by_you"
      ? "green"
      : ["refused", "refused_by_you", "refused_but_ran", "ran_without_asking", "not_offered", "turned_away"].includes(c.kind)
        ? "red"
        : c.kind === "not_judged"
          ? "amber"
          : "note";
  // Rust says when another model is on offer (E1): a model refused, and the run did not finish.
  const offersSwitch = $derived(report.offers_model_switch);
</script>

<article class="report" aria-label="The report">
  <section class="outcome {end.icon}" aria-labelledby="outcome-title">
    <p class="kicker">The report</p>
    <h1 id="outcome-title"><Icon kind={end.icon} size={24} />{end.title}</h1>
    <p class="end-text">{end.text}</p>
    {#if report.end.kind === "stopped_by_steplet"}
      <div class="printed"><FileText text={report.end.about} block /></div>
    {/if}
    {#if offersSwitch}
      {#each report.refusals as r, i (i)}
        <div class="message warn">
          <p>
            <span class="mono"><FileText text={r.model} /></span> refused. The kind of request it refused:
            <FileText text={r.category} />.
          </p>
          <div class="quote"><FileText text={r.explanation} block /></div>
          <p>Steplet keeps the run on the model you approved. You can run it again and let Claude Code use another one.</p>
        </div>
      {/each}
    {/if}
    {#if !report.completed && report.faults.length > 0}
      <ul class="faults" aria-label="Why it did not finish">
        {#each report.faults as f, i (i)}<li><Icon kind="red" /><FileText text={f} /></li>{/each}
      </ul>
    {/if}
    <div class="actions">
      <button class="button primary" type="button" disabled={saving} onclick={save}>Save report…</button>
      {#if offersSwitch}
        <button class="button" type="button" onclick={() => onrun(true)}>Run again, allowing another model</button>
      {/if}
      <button class="button" type="button" onclick={() => onrun(false)}>Run again</button>
      <button class="button" type="button" onclick={onopen}>Open another file</button>
    </div>
    {#if savedAs}
      <p class="saved" role="status"><Icon kind="green" /><span>Saved as {savedAs}</span></p>
    {/if}
    {#if saveError}
      <p class="message error" role="alert">{saveError}</p>
    {/if}
  </section>

  <section class="panel" aria-labelledby="commands-title">
    <h2 id="commands-title">The commands</h2>
    {#if report.commands.length === 0}
      <p class="muted">The agent ran none.</p>
    {:else}
      <ol class="commands">
        {#each report.commands as c, i (i)}
          <li class="command">
            <div class="call-head">
              <span class="tool"><FileText text={c.tool} /></span>
              <span class="what"><FileText text={c.command} block /></span>
            </div>
            <div class="call-facts">
              <span class="check {checkIcon(c.check)}"><Icon kind={checkIcon(c.check)} />{c.check.words}</span>
              <span class="exit" class:failed={c.failed}>{c.ended}</span>
            </div>
            {#if c.after_stop}
              <p class="muted">Steplet was already stopping the run: this was cut short or refused by the stop.</p>
            {/if}
            {#if c.lines > 0}
              <div class="output">
                <FileText text={c.output} block />
              </div>
            {:else}
              <p class="muted">It printed nothing.</p>
            {/if}
          </li>
        {/each}
      </ol>
    {/if}
  </section>

  <section class="panel" aria-labelledby="refused-title">
    <h2 id="refused-title">What Steplet refused</h2>
    {#if report.refused.length === 0}
      <p class="muted">Nothing.</p>
    {:else}
      <ul class="items">
        {#each report.refused as r, i (i)}<li class="mono"><FileText text={r} /></li>{/each}
      </ul>
    {/if}
  </section>

  <section class="panel" aria-labelledby="answers-title">
    <h2 id="answers-title">Your answers</h2>
    {#if report.answers.length === 0}
      <p class="muted">Steplet asked you nothing.</p>
    {:else}
      <ul class="items">
        {#each report.answers as a, i (i)}
          <li>
            <span class="mono"><FileText text={a.tool} />: <FileText text={a.what} /></span>
            <span class="reply">{a.reply}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="panel" aria-labelledby="models-title">
    <h2 id="models-title">The models</h2>
    {#if report.spent}
      {@const s = report.spent}
      <dl class="spend">
        <dt>The model</dt>
        <dd>{s.model}</dd>
        <dt>Spent</dt>
        <dd>{s.money}</dd>
      </dl>
      <p class="muted">{s.tokens}</p>
      {#if s.unsure}<p>{s.unsure}</p>{/if}
    {/if}
    {#if report.models.length === 0}
      <p class="muted">None recorded.</p>
    {:else}
      <p>
        Used:
        {#each report.models as m, i (i)}<span class="chip"><FileText text={m} /></span>{/each}
      </p>
    {/if}
    {#each report.refusals as r, i (i)}
      <div class="message warn">
        <p>
          <span class="mono"><FileText text={r.model} /></span> refused. The kind of request it refused:
          <FileText text={r.category} />.
        </p>
        <div class="quote"><FileText text={r.explanation} block /></div>
        {#if r.switched_to}<p>Claude Code went on with <span class="mono"><FileText text={r.switched_to} /></span>.</p>{/if}
      </div>
    {/each}
  </section>

  <section class="panel" aria-labelledby="changed-title">
    <h2 id="changed-title">The files the run changed</h2>
    <p class="muted">In its working copy, which Steplet deleted after the run.</p>
    {#if report.changed_files === null}
      <p>Steplet could not look: what went wrong around the run says why.</p>
    {:else if report.changed_files.length === 0}
      <p class="muted">None.</p>
    {:else}
      <ul class="items">
        {#each report.changed_files as f, i (i)}<li class="mono"><FileText text={f} /></li>{/each}
      </ul>
    {/if}
  </section>

  {#if report.trust !== undefined}
    <section class="panel" aria-labelledby="trust-title">
      <h2 id="trust-title">The trust check</h2>
      <p>{report.trust}</p>
    </section>
  {/if}

  <section class="panel" aria-labelledby="faults-title">
    <h2 id="faults-title">What went wrong around the run</h2>
    {#if report.faults.length === 0}
      <p class="muted">Nothing.</p>
    {:else}
      <ul class="items">
        {#each report.faults as f, i (i)}<li><FileText text={f} /></li>{/each}
      </ul>
    {/if}
  </section>

  <section class="panel" aria-labelledby="notes-title">
    <h2 id="notes-title">Steplet's notes</h2>
    <ul class="items">
      {#each report.notes as n, i (i)}<li>{n}</li>{/each}
    </ul>
  </section>

  <section class="panel" aria-labelledby="says-title">
    <h2 id="says-title">The agent says</h2>
    <p class="muted">Its own words, not Steplet's: they say nothing about whether the run went well.</p>
    {#if report.agent_says.length > 0}
      <div class="quote"><FileText text={report.agent_says} block /></div>
    {:else}
      <p class="muted">Nothing.</p>
    {/if}
  </section>
</article>

<style>
  .report {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-bottom: var(--space-12);
  }

  .outcome {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-5) var(--space-6);
    border: 1px solid var(--border);
    border-left-width: 4px;
    border-radius: var(--radius-panel);
    background: var(--surface);
  }

  .outcome.green {
    border-left-color: var(--green-fg);
  }

  .outcome.amber {
    border-left-color: var(--amber-border);
  }

  .outcome.red {
    border-left-color: var(--red-fg);
  }

  .kicker {
    color: var(--text-muted);
  }

  h1 {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: calc(-1 * var(--space-2));
    font-family: var(--font-display);
    font-size: 24px;
    font-weight: 600;
    line-height: 32px;
  }

  .end-text {
    font-size: 15px;
  }

  h2 {
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    line-height: 22px;
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-top: var(--space-1);
  }

  .saved {
    display: flex;
    gap: var(--space-2);
    align-items: start;
    color: var(--green-fg);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .saved :global(.icon),
  .faults :global(.icon) {
    flex: none;
    margin-top: 2px;
  }

  .faults {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .faults li {
    display: flex;
    gap: var(--space-2);
    align-items: start;
    font-weight: 600;
  }

  .faults :global(.icon) {
    color: var(--red-fg);
  }

  .commands {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .command {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
    padding: var(--space-3) 0;
    border-top: 1px solid var(--border);
  }

  .command:first-child {
    padding-top: 0;
    border-top: 0;
  }

  .call-head {
    display: flex;
    gap: var(--space-2);
    align-items: baseline;
    min-width: 0;
  }

  .tool,
  .chip {
    flex: none;
    padding: 0 var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-sunken);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 20px;
  }

  .tool {
    color: var(--text-muted);
  }

  .chip {
    display: inline-block;
    margin: 0 var(--space-1) var(--space-1) 0;
  }

  .spend {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: var(--space-1) var(--space-4);
    margin: 0;
  }

  .spend dt {
    color: var(--text-muted);
  }

  .spend dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }

  .what {
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
  }

  .call-facts {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-3);
  }

  .check {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0 var(--space-2);
    border-radius: var(--radius);
    font-weight: 600;
    line-height: 24px;
  }

  .check.green {
    background: var(--green-bg);
    color: var(--green-fg);
  }

  .check.red {
    background: var(--red-bg);
    color: var(--red-fg);
  }

  .check.amber {
    background: var(--amber-bg);
    color: var(--amber-fg);
  }

  .check.note {
    background: var(--surface-sunken);
    color: var(--text-muted);
  }

  .exit {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  /* An exit code other than 0, from Rust's `failed`. */
  .exit.failed {
    color: var(--red-fg);
    font-weight: 600;
  }

  .output,
  .printed,
  .quote {
    max-width: 100%;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
  }

  .output,
  .printed {
    max-height: 320px;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 18px;
  }

  .items {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding-left: var(--space-5);
  }

  .items li {
    overflow-wrap: anywhere;
  }

  .reply {
    display: block;
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .muted {
    color: var(--text-muted);
  }

  .message {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: 1px solid;
    border-radius: var(--radius-panel);
  }

  .warn {
    border-color: var(--amber-border);
    background: var(--amber-bg);
    color: var(--amber-fg);
  }

  .warn .quote {
    color: var(--text);
  }

  .error {
    border-color: var(--red-border);
    background: var(--red-bg);
    color: var(--red-fg);
  }
</style>
