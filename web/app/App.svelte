<script lang="ts">
  // The app's one window. Task 8: the empty Open screen; the screens behind it come with Tasks 10 to 13. The backend
  // is in the context under "backend" (main.ts).
</script>

<div class="window">
  <header class="bar">
    <button class="link" type="button">Setup</button>
  </header>
  <main class="middle">
    <section class="open" aria-labelledby="open-title">
      <svg class="mark" viewBox="0 0 48 48" width="48" height="48" aria-hidden="true">
        <rect class="mark-square" width="48" height="48" rx="10.5" />
        <path class="mark-steps" d="M10.9 36.4V27.6h8.7v-8.7h8.7v-8.8h8.8v26.3z" />
      </svg>
      <h1 id="open-title">Open a .steplet file to see what it asks for.</h1>
      <p class="muted">Nothing in it runs until you press Run.</p>
      <div class="actions">
        <button class="button primary" type="button">Open a file…</button>
        <span class="muted">or drop it on this window</span>
      </div>
    </section>
  </main>
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .bar {
    display: flex;
    justify-content: flex-end;
    width: 100%;
    max-width: calc(var(--column) + 2 * var(--space-6));
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) 0;
  }

  .middle {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
  }

  .open {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    max-width: var(--column);
    padding: var(--space-12) var(--space-6);
    border: 1px dashed var(--control-border);
    border-radius: var(--radius-panel);
    background: var(--surface);
    text-align: center;
  }

  .mark {
    margin-bottom: var(--space-2);
  }

  .mark-square {
    fill: var(--accent);
  }

  .mark-steps {
    fill: var(--on-accent);
  }

  h1 {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 28px;
  }

  .muted {
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    margin-top: var(--space-4);
  }
</style>
