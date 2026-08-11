<script lang="ts">
  import Search from "@lucide/svelte/icons/search";
  import { query } from "$lib/stores/filters";
</script>

<label class="relative block flex-1">
  <span class="sr-only">Search ports and processes</span>
  <Search
    size={14}
    strokeWidth={1.8}
    class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
  />
  <input
    type="search"
    bind:value={$query}
    placeholder="Search ports or processes"
    class="h-8 w-full rounded-md border border-[var(--border-subtle)] bg-[var(--surface-muted)] pl-8 pr-16 text-[11px] text-[var(--text-primary)] placeholder:text-[var(--text-muted)]"
  />
  {#if $query.trim()}
    <span class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 rounded bg-[var(--primary-soft)] px-1.5 py-0.5 text-[9px] font-semibold text-[var(--primary)]">
      All ports
    </span>
  {/if}
</label>
