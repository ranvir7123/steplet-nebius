<script lang="ts">
  // A list of findings, worst first as the core ordered them: each with its severity as a word and an icon, its
  // title, the line it is about, Steplet's detail, the file's text it quotes, and what to do.
  import FileText from "./FileText.svelte";
  import Icon from "./Icon.svelte";
  import type { FindingView } from "./types/FindingView";

  let { findings }: { findings: FindingView[] } = $props();

  const word: Record<string, string> = { blocked: "Blocked", red: "Red", amber: "Amber", note: "Note" };
  const icon = (s: string) => (s === "blocked" || s === "red" || s === "amber" ? s : "note");
</script>

<ul class="findings">
  {#each findings as f, i (i)}
    <li class="finding">
      <p class="head">
        <span class="severity {f.severity}"><Icon kind={icon(f.severity)} />{word[f.severity] ?? f.severity}</span>
        <span class="title">{f.title}</span>
      </p>
      <p class="where">{f.line === null ? "About the whole file" : `Line ${f.line}`}</p>
      {#if f.detail}<p class="detail">{f.detail}</p>{/if}
      {#if f.quote}
        <div class="quote">
          <p class="quote-label">From the file</p>
          <FileText text={f.quote} block />
        </div>
      {/if}
      <p class="advice">{f.advice}</p>
    </li>
  {/each}
</ul>

<style>
  .findings {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .finding {
    padding: var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--surface);
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-2) var(--space-3);
  }

  .severity {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: var(--space-1);
    padding: 0 var(--space-2);
    border-radius: var(--radius);
    font-weight: 600;
    line-height: 24px;
    align-self: flex-start;
  }

  .severity.blocked {
    background: var(--blocked-bg);
    color: var(--blocked-fg);
  }

  .severity.red {
    background: var(--red-bg);
    color: var(--red-fg);
  }

  .severity.amber {
    background: var(--amber-bg);
    color: var(--amber-fg);
  }

  .severity.note {
    background: var(--surface-sunken);
    color: var(--text-muted);
  }

  .title {
    flex: 1 1 240px;
    font-weight: 600;
  }

  .where,
  .detail {
    margin-top: var(--space-1);
    color: var(--text-muted);
  }

  .quote {
    margin-top: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-left: 3px solid var(--file-rule);
    background: var(--file-bg);
  }

  .quote-label {
    margin-bottom: var(--space-1);
    color: var(--file-label);
    font-size: 12px;
    line-height: 16px;
  }

  .advice {
    margin-top: var(--space-3);
  }
</style>
