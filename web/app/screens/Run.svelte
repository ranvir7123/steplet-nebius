<script lang="ts">
  // Screens 5 and 6: prepare, then the run with its live log. A bar that stays at the top says where the run is, how
  // long it has run against its limit, and holds Stop (which asks first). One row per thing that happens: what the
  // agent says (quoted, as file text), each command or file action with Steplet's check, its exit code and its output
  // (the first 40 lines, then "Show all"), each question and how it ended. The log follows the newest row unless the
  // receiver scrolled up. Steplet's words come from Rust; everything the agent, the repository or a command wrote is
  // file text.
  import { getContext, onDestroy, onMount, tick } from "svelte";
  import FileText from "../../shared/FileText.svelte";
  import Icon from "../../shared/Icon.svelte";
  import type { CheckView } from "../../shared/types/CheckView";
  import type { EventView } from "../../shared/types/EventView";
  import type { FailureView } from "../../shared/types/FailureView";
  import type { PreviewView } from "../../shared/types/PreviewView";
  import type { ProgressView } from "../../shared/types/ProgressView";
  import type { QuestionView } from "../../shared/types/QuestionView";
  import type { ReportView } from "../../shared/types/ReportView";
  import type { Backend, Shown, Unlisten } from "../lib/backend";
  import { endWords } from "../lib/end";
  import { modal } from "../lib/modal";
  import { firstLines } from "../lib/shown";
  import Ask from "./Ask.svelte";

  let {
    view,
    typed,
    onback,
    onopen,
    onbusy,
  }: {
    view: PreviewView;
    /** The red file's sentence, for Rust to check again. */
    typed: string | null;
    onback: () => void;
    onopen: () => void;
    /** Whether a run goes on: the window then offers no other file and no Setup. */
    onbusy: (busy: boolean) => void;
  } = $props();

  const backend = getContext<Backend>("backend");
  const SHOWN_LINES = 40;

  interface Result {
    is_error: boolean;
    exit_code: number | null;
    output: Shown;
    lines: number;
  }
  type Row =
    | { kind: "started"; model: Shown }
    | { kind: "says"; text: Shown }
    | { kind: "call"; id: string; tool: Shown; what: Shown; check: CheckView | null; reason: string | null; result: Result | null; all: boolean }
    | { kind: "asked"; question: QuestionView; how: string | null }
    | { kind: "refused"; from: Shown; to: Shown | null; category: Shown; explanation: Shown }
    | { kind: "hook"; name: Shown }
    | { kind: "ended"; models: Shown[] }
    | { kind: "note"; text: Shown }
    | { kind: "retry"; words: string };

  type Phase = "preparing" | "running" | "stopping" | "finished" | "failed";
  let phase = $state<Phase>("preparing");
  let rows = $state<Row[]>([]);
  let prepared = $state<{ commit: string; removed: Shown[]; limit_ms: number } | null>(null);
  let question = $state<QuestionView | null>(null);
  let waiting = $state(0);
  let report = $state<ReportView | null>(null);
  let failed = $state<{ stage: string; failure: FailureView } | null>(null);
  let confirm = $state<"stop" | "close" | null>(null);
  let closing = $state(false);
  let startedAt = 0;
  let now = $state(0);
  // Checks that came before their call's row (the gate and the event stream are read on two threads).
  const early = new Map<string, { check: CheckView; reason: string | null }>();

  const going = $derived(phase === "preparing" || phase === "running" || phase === "stopping");
  $effect(() => onbusy(going));

  const callRow = (id: string) =>
    rows.find((r): r is Extract<Row, { kind: "call" }> => r.kind === "call" && r.id === id);

  function event(e: EventView) {
    switch (e.kind) {
      case "started":
        rows.push({ kind: "started", model: e.model });
        break;
      case "agent_says":
        rows.push({ kind: "says", text: e.text });
        break;
      case "call": {
        const seen = early.get(e.id);
        early.delete(e.id);
        rows.push({ kind: "call", id: e.id, tool: e.tool, what: e.what, check: seen?.check ?? null, reason: seen?.reason ?? null, result: null, all: false });
        break;
      }
      case "result": {
        const result = { is_error: e.is_error, exit_code: e.exit_code, output: e.output, lines: e.lines };
        const row = callRow(e.id);
        if (row) row.result = result;
        else rows.push({ kind: "call", id: e.id, tool: [], what: [], check: null, reason: null, result, all: false });
        break;
      }
      case "skipped":
        // Its check comes as its own progress (Rust's words); the row is the call's, if there was one.
        if (!callRow(e.id)) rows.push({ kind: "call", id: e.id, tool: e.tool, what: e.what, check: null, reason: null, result: null, all: false });
        break;
      case "model_refused":
        rows.push({ kind: "refused", from: e.from, to: e.to, category: e.category, explanation: e.explanation });
        break;
      case "hook":
        rows.push({ kind: "hook", name: e.name });
        break;
      case "ended":
        rows.push({ kind: "ended", models: e.models });
        break;
      case "note":
        rows.push({ kind: "note", text: e.text });
        break;
      case "retry": {
        // One row that counts the tries, not one row per try.
        const last = rows[rows.length - 1];
        if (last?.kind === "retry") last.words = e.words;
        else rows.push({ kind: "retry", words: e.words });
        break;
      }
    }
  }

  function take(p: ProgressView) {
    switch (p.kind) {
      case "prepared":
        prepared = { commit: p.commit, removed: p.removed, limit_ms: p.limit_ms };
        startedAt = performance.now();
        now = startedAt;
        if (phase === "preparing") phase = "running";
        break;
      case "event":
        event(p.event);
        break;
      case "question":
        question = p.question;
        rows.push({ kind: "asked", question: p.question, how: null });
        break;
      case "question_closed": {
        const row = rows.find((r) => r.kind === "asked" && r.question.id === p.id);
        if (row?.kind === "asked") row.how = p.how;
        if (question?.id === p.id) question = null;
        break;
      }
      case "waiting":
        waiting = p.count;
        break;
      case "checked": {
        const row = callRow(p.call);
        if (row) {
          row.check = p.check;
          row.reason = p.reason;
        } else early.set(p.call, { check: p.check, reason: p.reason });
        break;
      }
      case "finished":
        report = p.report;
        question = null;
        confirm = null;
        phase = "finished";
        break;
      case "failed":
        failed = { stage: p.stage, failure: p.failure };
        question = null;
        confirm = null;
        phase = "failed";
        break;
    }
  }

  let unlisten: Unlisten[] = [];
  const clock = setInterval(() => {
    if (prepared && going) now = performance.now();
  }, 1000);

  async function start() {
    rows = [];
    prepared = null;
    question = null;
    waiting = 0;
    report = null;
    failed = null;
    early.clear();
    phase = "preparing";
    following = true;
    try {
      await backend.startRun(view.sha256, typed);
    } catch (e) {
      // Rust refused to start: its own sentence (the file changed, expired, another run goes on, …).
      take({
        kind: "failed",
        stage: "Starting the run",
        failure: { reason: typeof e === "string" ? e : "Steplet could not start the run.", detail: null },
      });
    }
  }

  onMount(() => {
    void (async () => {
      unlisten.push(await backend.onRunProgress(take));
      unlisten.push(await backend.onCloseAsked(() => (going ? (confirm = "close") : undefined)));
      await start();
    })();
    window.scrollTo(0, 0);
  });

  onDestroy(() => {
    clearInterval(clock);
    for (const u of unlisten) u();
    onbusy(false);
  });

  function answer(yes: boolean) {
    const q = question;
    if (!q) return;
    void backend.answer(q.id, yes, q.fingerprint);
  }

  async function stop() {
    confirm = null;
    phase = "stopping";
    await backend.stopRun();
  }

  async function stopAndClose() {
    confirm = null;
    closing = true;
    if (going) phase = "stopping";
    await backend.stopAndClose();
  }

  // Following the newest row: only the receiver's own scrolling up stops it, and scrolling back to the end starts it
  // again. The log's own growth never does.
  let following = $state(true);
  let lastY = 0;
  const atEnd = () => window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 2;
  function scrolled() {
    if (window.scrollY < lastY - 1) following = false;
    else if (atEnd()) following = true;
    lastY = window.scrollY;
  }
  function toEnd() {
    window.scrollTo(0, document.documentElement.scrollHeight);
    lastY = window.scrollY;
  }
  $effect(() => {
    // Every change to the log, and the end, move the page when it follows.
    void rows.length;
    for (const r of rows) if (r.kind === "call") void [r.result, r.check, r.all];
    void [phase, failed, report];
    if (!following) return;
    void tick().then(() => following && toEnd());
  });
  function newest() {
    following = true;
    toEnd();
  }

  const elapsed = $derived(prepared ? Math.max(0, now - startedAt) : 0);
  const mmss = (ms: number) => {
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  };
  const end = $derived(report ? endWords(report.end) : null);
  const lastSaid = $derived.by(() => {
    for (let i = rows.length - 1; i >= 0; i--) {
      const r = rows[i];
      if (r.kind === "says") return r.text;
    }
    return null;
  });
  const status = $derived.by((): { word: string; icon: "green" | "amber" | "red" | "note" } => {
    if (closing) return { word: "Closing", icon: "note" };
    switch (phase) {
      case "preparing":
        return { word: "Preparing", icon: "note" };
      case "running":
        return question ? { word: "Waiting for your answer", icon: "amber" } : { word: "Running", icon: "note" };
      case "stopping":
        return { word: "Stopping…", icon: "note" };
      case "finished":
        return { word: end?.title ?? "Finished", icon: end?.icon ?? "green" };
      case "failed":
        return { word: "Nothing ran", icon: "red" };
    }
  });
  const checkIcon = (c: CheckView): "green" | "amber" | "red" | "note" =>
    c.kind === "allowed" || c.kind === "allowed_by_you"
      ? "green"
      : c.kind === "refused" || c.kind === "refused_by_you" || c.kind === "refused_but_ran" || c.kind === "ran_without_asking"
        ? "red"
        : c.kind === "not_judged"
          ? "amber"
          : "note";
  /** Refused at the gate: it never ran, so there is no exit to tell. */
  const refusedBeforeRunning = (c: CheckView | null) => c?.kind === "refused" || c?.kind === "refused_by_you";
  const repository = $derived(view.file?.repository ?? null);
  const count = (n: number) => n.toLocaleString("en-US");
</script>

<svelte:window onscroll={scrolled} />

<article class="run" aria-label="The run">
  <div class="bar">
    <p class="state"><Icon kind={status.icon} /><span>{status.word}</span></p>
    {#if prepared}
      <p class="time">
        <span class="muted">Time</span>
        {mmss(elapsed)} <span class="muted">of {mmss(prepared.limit_ms)}</span>
      </p>
    {/if}
    {#if going && !closing}
      <button class="button stop" type="button" disabled={phase === "stopping"} onclick={() => (confirm = "stop")}>Stop</button>
    {/if}
  </div>
  {#if closing}
    <p class="closing" role="status">Steplet is stopping the run, then it closes.</p>
  {/if}

  {#if prepared || !failed}
    <section class="panel prepare" aria-labelledby="prepare-title">
      <h2 id="prepare-title">Getting the repository ready</h2>
      <ul class="steps">
        <li class="step">
          <Icon kind={prepared ? "green" : "note"} />
          <span>
            {prepared ? "Fetched" : "Fetching"}
            {#if repository}<span class="mono"><FileText text={repository.url.text} /></span>{:else}the repository{/if}
          </span>
        </li>
        <li class="step">
          <Icon kind={prepared ? "green" : "note"} />
          <span>
            {prepared ? "Checked out commit" : "Checking out commit"}
            {#if prepared}<span class="mono">{prepared.commit}</span>{:else if repository}<span class="mono"
                ><FileText text={repository.commit.text} /></span
              >{/if}
            inside the fence
          </span>
        </li>
        {#if prepared}
          <li class="step">
            <Icon kind="green" />
            <span>
              {#if prepared.removed.length > 0}
                Removed the repository's agent settings files:
                {#each prepared.removed as r, i (i)}<span class="mono"><FileText text={r} /></span>{i < prepared.removed.length - 1 ? ", " : ""}{/each}.
              {:else}
                The repository carries no agent settings files.
              {/if}
              Steplet tells the agent not to load agent settings.
            </span>
          </li>
        {/if}
      </ul>
      {#if phase === "preparing" && !failed}
        <p class="muted" role="status">Steplet is fetching the repository into its own Linux…</p>
      {/if}
    </section>
  {/if}

  {#if rows.length > 0}
    <ol class="log" role="log" aria-label="What happens in the run">
      {#each rows as r, i (i)}
        <li class="row {r.kind}">
          {#if r.kind === "started"}
            <p class="muted">Claude Code started, with the model <span class="mono"><FileText text={r.model} /></span>.</p>
          {:else if r.kind === "says"}
            <div class="quote">
              <p class="quote-label">The agent says</p>
              <FileText text={r.text} block />
            </div>
          {:else if r.kind === "call"}
            <div class="call-head">
              {#if r.tool.length > 0}<span class="tool"><FileText text={r.tool} /></span>{/if}
              <span class="what"><FileText text={r.what} block /></span>
            </div>
            <div class="call-facts">
              {#if r.check}
                <span class="check {checkIcon(r.check)}"><Icon kind={checkIcon(r.check)} />{r.check.words}</span>
              {:else}
                <span class="check note"><Icon kind="note" />Steplet is checking it</span>
              {/if}
              {#if r.reason}<span class="reason">Steplet's reason: {r.reason}</span>{/if}
              {#if r.result && !refusedBeforeRunning(r.check)}
                <span class="exit">
                  {#if r.result.exit_code !== null}Exit code {r.result.exit_code}{:else if r.result.is_error}Ended with an error{:else}Done{/if}
                </span>
              {/if}
            </div>
            {#if r.result && r.result.lines > 0}
              {@const result = r.result}
              <section class="output" aria-label="What it printed">
                <FileText text={r.all ? result.output : firstLines(result.output, SHOWN_LINES)} block />
              </section>
              {#if result.lines > SHOWN_LINES}
                <button class="link more" type="button" onclick={() => (r.all = !r.all)}>
                  {r.all ? `Show the first ${SHOWN_LINES} lines` : `Show all ${count(result.lines)} lines`}
                </button>
              {/if}
            {/if}
          {:else if r.kind === "asked"}
            <p class="asked">
              <Icon kind="amber" />
              <span>
                Steplet asked you: {r.question.ask}
                <span class="mono"><FileText text={r.question.what} /></span>
              </span>
            </p>
            <p class="how">{r.how ?? "Waiting for your answer."}</p>
          {:else if r.kind === "refused"}
            <div class="message warn">
              <p>
                The model <span class="mono"><FileText text={r.from} /></span> refused. The kind of request it refused:
                <FileText text={r.category} />.
              </p>
              <FileText text={r.explanation} block />
              {#if r.to}<p>Claude Code went on with the model <span class="mono"><FileText text={r.to} /></span>.</p>{/if}
            </div>
          {:else if r.kind === "hook"}
            <p class="muted">Claude Code ran a hook: <span class="mono"><FileText text={r.name} /></span></p>
          {:else if r.kind === "ended"}
            <p class="muted">
              The agent stopped working.
              {#if r.models.length > 0}Models used:
                {#each r.models as m, j (j)}<span class="mono"><FileText text={m} /></span>{j < r.models.length - 1 ? ", " : ""}{/each}.{/if}
            </p>
          {:else if r.kind === "retry"}
            <p class="retry"><Icon kind="amber" /><span>{r.words}</span></p>
          {:else}
            <div class="printed-by">
              <p class="quote-label">Claude Code printed</p>
              <FileText text={r.text} block />
            </div>
          {/if}
        </li>
      {/each}
    </ol>
  {/if}

  {#if failed}
    <section class="message error" aria-labelledby="failed-title" role="alert">
      <h2 id="failed-title">{failed.stage}</h2>
      <p>{failed.failure.reason}</p>
      {#if failed.failure.detail}<div class="printed"><FileText text={failed.failure.detail} block /></div>{/if}
      <div class="actions">
        <button class="button primary" type="button" onclick={start}>Try again</button>
        <button class="button" type="button" onclick={onback}>Back</button>
      </div>
    </section>
  {/if}

  {#if report && end}
    <section class="panel end {end.icon}" aria-labelledby="end-title">
      <h2 id="end-title" class="end-title"><Icon kind={end.icon} size={20} />{end.title}</h2>
      <p>{end.text}</p>
      {#if report.end.kind === "stopped_by_steplet"}<div class="printed"><FileText text={report.end.about} block /></div>{/if}
      {#if report.faults.length > 0}
        <ul class="faults" aria-label="What went wrong around the run">
          {#each report.faults as f, i (i)}<li><Icon kind="red" /><FileText text={f} /></li>{/each}
        </ul>
      {/if}
      {#if !closing}
        <div class="actions">
          <button class="button primary" type="button" onclick={start}>Run again</button>
          <button class="button" type="button" onclick={onopen}>Open another file</button>
        </div>
      {/if}
    </section>
  {/if}

  {#if !following && rows.length > 0}
    <button class="button newest" type="button" onclick={newest}>Show the newest</button>
  {/if}
</article>

{#if question}
  {#key question.id}
    <Ask {question} {waiting} said={lastSaid} onanswer={answer} />
  {/key}
{/if}

{#if confirm}
  <dialog class="confirm" aria-labelledby="confirm-title" use:modal>
    {#if confirm === "stop"}
      <h2 id="confirm-title">Stop the run?</h2>
      <p>Steplet stops the agent at once and resets the run's folder. What the agent did so far stays in the report.</p>
    {:else}
      <h2 id="confirm-title">Close Steplet?</h2>
      <p>A file is running. Steplet stops the run first and resets its folder, then it closes.</p>
    {/if}
    <div class="actions">
      <button class="button primary" type="button" data-first onclick={() => (confirm = null)}>Keep running</button>
      {#if confirm === "stop"}
        <button class="button" type="button" onclick={stop}>Stop the run</button>
      {:else}
        <button class="button" type="button" onclick={stopAndClose}>Stop and close</button>
      {/if}
    </div>
  </dialog>
{/if}

<style>
  .run {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-bottom: var(--space-12);
  }

  .bar {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-5);
    min-height: 56px;
    margin-top: calc(-1 * var(--space-6));
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
  }

  .state {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    line-height: 22px;
  }

  .time {
    font-variant-numeric: tabular-nums;
  }

  .stop {
    margin-left: auto;
  }

  .closing {
    font-weight: 600;
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

  .steps {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .step {
    display: grid;
    grid-template-columns: 16px minmax(0, 1fr);
    gap: var(--space-2);
    align-items: start;
  }

  .step :global(.icon) {
    margin-top: 2px;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .log {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
    list-style: none;
  }

  .row {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
    padding: var(--space-3) var(--space-5);
    border-top: 1px solid var(--border);
  }

  .row:first-child {
    border-top: 0;
  }

  .quote,
  .printed-by {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
  }

  .quote-label {
    color: var(--file-label);
    font-size: 12px;
    font-weight: 600;
    line-height: 16px;
  }

  .printed-by {
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .printed-by .quote-label {
    font-family: var(--font-text);
  }

  .call-head {
    display: flex;
    gap: var(--space-2);
    align-items: baseline;
    min-width: 0;
  }

  .tool {
    flex: none;
    padding: 0 var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-sunken);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 20px;
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

  .reason,
  .exit {
    color: var(--text-muted);
  }

  .exit {
    font-variant-numeric: tabular-nums;
  }

  .output,
  .printed {
    max-width: 100%;
    overflow-x: auto;
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 18px;
  }

  .more {
    align-self: flex-start;
  }

  .asked {
    display: flex;
    gap: var(--space-2);
    align-items: start;
    font-weight: 600;
  }

  .asked :global(.icon) {
    flex: none;
    margin-top: 2px;
  }

  .retry {
    display: flex;
    gap: var(--space-2);
    align-items: start;
    color: var(--amber-fg);
  }

  .retry :global(.icon) {
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
    flex: none;
    margin-top: 2px;
    color: var(--red-fg);
  }

  .how {
    padding-left: calc(16px + var(--space-2));
    color: var(--text-muted);
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

  .error {
    border-color: var(--red-border);
    background: var(--red-bg);
    color: var(--red-fg);
  }

  .error .printed,
  .warn :global(.file-text.block) {
    color: var(--text);
  }

  .end-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 18px;
    line-height: 24px;
  }

  .end.green {
    border-color: var(--green-border);
  }

  .end.amber {
    border-color: var(--amber-border);
  }

  .end.red {
    border-color: var(--red-border);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .newest {
    position: fixed;
    bottom: var(--space-6);
    left: 50%;
    z-index: 2;
    transform: translateX(-50%);
    box-shadow: 0 4px 16px rgb(0 0 0 / 20%);
  }

  .confirm {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    width: min(480px, calc(100vw - 2 * var(--space-4)));
    padding: var(--space-5) var(--space-6);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 16px 48px rgb(0 0 0 / 28%);
  }

  .confirm::backdrop {
    background: rgb(0 0 0 / 45%);
  }

  .confirm h2 {
    font-size: 18px;
    line-height: 24px;
  }

  .confirm .actions {
    margin-top: var(--space-2);
  }
</style>
