<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    title = label,
    id,
    disabled = false,
    danger = false,
    active = false,
    onclick,
    children,
  }: {
    label: string;
    title?: string;
    id?: string;
    disabled?: boolean;
    danger?: boolean;
    active?: boolean;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();
</script>

<button
  type="button"
  {id}
  aria-label={label}
  aria-pressed={active ? true : undefined}
  {title}
  {disabled}
  {onclick}
  class:text-[var(--danger)]={danger}
  class={`grid size-8 shrink-0 place-items-center rounded-md transition-colors duration-150 disabled:pointer-events-none disabled:opacity-40 ${
    active
      ? "bg-[var(--surface-muted)] text-[var(--text-primary)]"
      : "text-[var(--text-secondary)] hover:bg-[var(--surface-muted)] hover:text-[var(--text-primary)]"
  }`}
>
  {@render children()}
</button>
