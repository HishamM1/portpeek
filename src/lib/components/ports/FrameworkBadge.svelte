<script lang="ts">
  import type { FrameworkDetection } from "$lib/types/framework";

  let { framework }: { framework: FrameworkDetection | null } = $props();
</script>

{#if framework}
  <span
    title={`${framework.confidence} confidence · ${framework.source.replaceAll("_", " ")}`}
    class="rounded border border-[var(--primary-border)] bg-[var(--primary-soft)] px-1.5 py-0.5 text-[10px] font-semibold text-[var(--primary)]"
  >
    {framework.name}
  </span>
{/if}
