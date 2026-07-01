<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<section
  aria-label="PortPeek"
  class="flex h-screen flex-col overflow-hidden bg-[var(--surface)] [animation:popup-in_160ms_cubic-bezier(.2,.8,.2,1)]"
>
  {@render children()}
</section>
