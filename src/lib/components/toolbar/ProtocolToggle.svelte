<script lang="ts">
  import { showUdp } from "$lib/stores/filters";
</script>

<button
  type="button"
  aria-pressed={$showUdp}
  title={$showUdp ? "Showing UDP sockets — click to hide" : "Show UDP sockets (hidden by default)"}
  onclick={() => ($showUdp = !$showUdp)}
  class={`flex h-8 shrink-0 items-center gap-1.5 rounded-md border px-2 text-[10px] font-medium transition-colors ${
    $showUdp
      ? "border-[var(--primary-border)] bg-[var(--primary-soft)] text-[var(--primary)]"
      : "border-[var(--border-subtle)] text-[var(--text-secondary)] hover:bg-[var(--surface-muted)]"
  }`}
>
  UDP
</button>
