<script lang="ts">
  // The only drawer of text from the file, the repository, the agent or a command (row 28): display pieces as plain
  // text, a badge for each character that shows nothing, and a link as its shown text with its real address beside
  // it. Never a clickable link, never HTML. Every piece carries data-from-file, so it can never pass for Steplet's
  // own words.
  import type { Part } from "./types/Part";
  import type { Seg } from "./types/Seg";

  let { text, block = false }: { text: Seg[]; block?: boolean } = $props();

  const code = (cp: number) => "U+" + cp.toString(16).toUpperCase().padStart(4, "0");
  const named = (cp: number) => (cp === 0x0a ? "A line break inside one entry" : "A character that shows nothing");
  const plain = (parts: Part[]) => parts.map((p) => (p.kind === "text" ? p.text : code(p.codepoint))).join("");
</script>

{#snippet badge(cp: number)}<span class="badge" title={named(cp)} aria-label="{named(cp)}, {code(cp)}">{code(cp)}</span>{/snippet}

{#snippet parts(list: Part[])}{#each list as p, i (i)}{#if p.kind === "text"}{p.text}{:else}{@render badge(p.codepoint)}{/if}{/each}{/snippet}

<span class="file-text" class:block data-from-file>{#each text as s, i (i)}{#if s.kind === "text"}{s.text}{:else if s.kind === "invisible"}{@render badge(s.codepoint)}{:else}<span class="file-link">{#if s.shown}{@render parts(s.shown)}{/if}<span class="real" class:after={!!s.shown || i > 0} class:before={i < text.length - 1} title="Where this link really goes: {plain(s.real)}"><span class="real-label">{s.shown ? "goes to" : "address"}</span>{@render parts(s.real)}</span></span>{/if}{/each}</span>

<style>
  .file-text {
    overflow-wrap: anywhere;
  }

  .block {
    display: block;
    white-space: pre-wrap;
  }

  .badge {
    display: inline-block;
    margin: 0 2px;
    padding: 0 4px;
    border-radius: var(--radius);
    background: var(--badge-bg);
    color: var(--badge-fg);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 18px;
    white-space: nowrap;
  }

  .real {
    display: inline-block;
    padding: 0 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-sunken);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 18px;
  }

  /* Space from what comes before (its shown text, or other text) and after; an address that starts the text stays in
     line with the text under it. */
  .after {
    margin-left: 4px;
  }

  .before {
    margin-right: 4px;
  }

  .real-label {
    margin-right: 4px;
    color: var(--text-muted);
    font-family: var(--font-text);
  }
</style>
