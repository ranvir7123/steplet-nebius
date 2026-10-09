<script lang="ts">
  import { onMount } from "svelte";
  import { runChecks, type CheckResults } from "./checks";

  let results: CheckResults | null = $state(null);
  let error = $state("");

  onMount(async () => {
    try {
      results = await runChecks();
      if ("__TAURI_INTERNALS__" in window) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("report", { text: JSON.stringify({ ...results, renderedBySvelte: true, userAgent: navigator.userAgent }) });
      }
    } catch (e) {
      error = String(e);
    }
  });
</script>

<main>
  <h1>Steplet spike S9</h1>
  {#if error}
    <p data-testid="error">{error}</p>
  {:else if !results}
    <p data-testid="loading">Checking…</p>
  {:else}
    <p data-testid="valid">Sample signature: {results.sample}</p>
    <p data-testid="tampered">Changed message: {results.tampered}</p>
    <p data-testid="fetch">Network request from this page: {results.fetchBlocked ? "blocked" : "NOT blocked"}</p>
  {/if}
</main>
