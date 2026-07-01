<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    disabled = false,
    danger = false,
    onclick,
    children,
  }: {
    disabled?: boolean;
    danger?: boolean;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();
</script>

<button
  type="button"
  {disabled}
  {onclick}
  class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md px-3 text-xs font-semibold text-[var(--text-inverse)] transition-colors disabled:pointer-events-none disabled:opacity-40"
  class:bg-[var(--danger)]={danger}
  class:hover:bg-[var(--danger)]={danger}
  class:bg-[var(--primary)]={!danger}
  class:hover:bg-[var(--primary-hover)]={!danger}
  class:active:bg-[var(--primary-active)]={!danger}
>
  {@render children()}
</button>
