<script lang="ts">
  // The outcome icons, drawn as inline SVG (not emoji, plan section 3.5): a tick, an exclamation mark, a cross and a
  // no-entry sign. Always beside a word, never alone; hidden from screen readers, which read the word.
  let { kind, size = 16 }: { kind: "green" | "amber" | "red" | "blocked" | "note"; size?: number } = $props();
</script>

<svg class="icon" viewBox="0 0 24 24" width={size} height={size} aria-hidden="true">
  {#if kind === "green"}
    <circle cx="12" cy="12" r="10" class="line" />
    <path d="M7.5 12.5l3 3 6-7" class="line" />
  {:else if kind === "amber"}
    <path d="M12 3.2L22 20.5H2z" class="line" />
    <path d="M12 9.5v5" class="line" />
    <circle cx="12" cy="17.6" r="1.1" class="dot" />
  {:else if kind === "red"}
    <circle cx="12" cy="12" r="10" class="line" />
    <path d="M8.5 8.5l7 7M15.5 8.5l-7 7" class="line" />
  {:else if kind === "blocked"}
    <circle cx="12" cy="12" r="10" class="line" />
    <path d="M6.5 12h11" class="bar" />
  {:else}
    <circle cx="12" cy="12" r="10" class="line" />
    <path d="M12 11v6" class="line" />
    <circle cx="12" cy="7.6" r="1.1" class="dot" />
  {/if}
</svg>

<style>
  .icon {
    flex: none;
  }

  .line {
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .bar {
    fill: none;
    stroke: currentColor;
    stroke-width: 3;
    stroke-linecap: round;
  }

  .dot {
    fill: currentColor;
  }
</style>
