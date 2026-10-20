<script lang="ts" module>
  export type Colour = "green" | "amber" | "red" | "blocked";

  /** Each colour's word and its one line of meaning (R9, R11): colour is never the only signal. */
  export const MEANING: Record<Colour, { word: string; line: string }> = {
    green: {
      word: "Green",
      line: "Nothing in this file asks for more than it declares, and everything declared is low risk.",
    },
    amber: { word: "Amber", line: "Continue only if you trust the sender." },
    red: { word: "Red", line: "Run only if you are sure: you will type a sentence first." },
    blocked: { word: "Blocked", line: "This file cannot run." },
  };
</script>

<script lang="ts">
  import Icon from "./Icon.svelte";

  let { colour }: { colour: Colour } = $props();
</script>

<section class="outcome {colour}" aria-label="How Steplet rates this file">
  <Icon kind={colour} size={24} />
  <div>
    <p class="word">{MEANING[colour].word}</p>
    <p class="line">{MEANING[colour].line}</p>
  </div>
</section>

<style>
  .outcome {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-5);
    border: 1px solid;
    border-radius: var(--radius-panel);
  }

  .word {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 24px;
  }

  .line {
    margin-top: var(--space-1);
  }

  .green {
    border-color: var(--green-border);
    background: var(--green-bg);
    color: var(--green-fg);
  }

  .amber {
    border-color: var(--amber-border);
    background: var(--amber-bg);
    color: var(--amber-fg);
  }

  .red {
    border-color: var(--red-border);
    background: var(--red-bg);
    color: var(--red-fg);
  }

  .blocked {
    border-color: var(--blocked-border);
    background: var(--blocked-bg);
    color: var(--blocked-fg);
  }
</style>
