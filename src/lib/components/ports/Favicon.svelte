<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import AppWindow from "@lucide/svelte/icons/app-window";
  import Database from "@lucide/svelte/icons/database";
  import type { PortItem } from "$lib/types/port";
  import { brandSlug, isDatabase } from "$lib/utils/ports.js";

  let {
    port,
    label,
  }: {
    port: PortItem;
    label: string;
  } = $props();
  let failed = $state(new Set<string>());
  let localSource = $derived(port.cachedFaviconPath ? convertFileSrc(port.cachedFaviconPath) : null);
  let brand = $derived(brandSlug(port));
  let brandSource = $derived(brand ? (brand.startsWith("https://") ? brand : `https://cdn.simpleicons.org/${brand}`) : null);
  let source = $derived([brandSource, localSource].find((candidate) => candidate && !failed.has(candidate)) ?? null);

  function markFailed(): void {
    if (source) failed = new Set([...failed, source]);
  }
</script>

<span
  class="grid size-10 shrink-0 place-items-center overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-white shadow-sm"
>
  {#if source}
    <img src={source} alt="" class="size-6 object-contain" referrerpolicy="no-referrer" onerror={markFailed} />
  {:else}
    {#if isDatabase(port)}
      <Database size={19} strokeWidth={1.6} class="text-[var(--text-muted)]" aria-hidden="true" />
    {:else}
      <AppWindow size={19} strokeWidth={1.6} class="text-[var(--text-muted)]" aria-hidden="true" />
    {/if}
  {/if}
</span>
