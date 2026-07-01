<script lang="ts">
  import type { PortProtocol } from "$lib/types/port";

  let { port, protocol = "tcp" }: { port: number; protocol?: PortProtocol } = $props();
</script>

{#if protocol === "udp"}
  <span
    class="inline-flex shrink-0 items-center gap-1 rounded-md border border-[var(--border-strong)] bg-[var(--surface-muted)] px-1.5 py-0.5 font-mono text-[12px] font-semibold tabular-nums text-[var(--text-secondary)]"
    >:{port}<span class="text-[9px] font-bold uppercase tracking-wide text-[var(--text-muted)]">udp</span></span
  >
{:else}
  <span
    class="shrink-0 rounded-md border border-[var(--primary-border)] bg-[var(--primary-soft)] px-1.5 py-0.5 font-mono text-[12px] font-semibold tabular-nums text-[var(--primary)]"
    >:{port}</span
  >
{/if}
