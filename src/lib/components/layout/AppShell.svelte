<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<main class="h-screen bg-[var(--surface)] text-[var(--text-primary)]">
  {@render children()}
</main>
