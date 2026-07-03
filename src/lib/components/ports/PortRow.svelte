<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Container from "@lucide/svelte/icons/container";
  import Globe from "@lucide/svelte/icons/globe";
  import MemoryStick from "@lucide/svelte/icons/memory-stick";
  import Terminal from "@lucide/svelte/icons/terminal";
  import { slide } from "svelte/transition";
  import Favicon from "$lib/components/ports/Favicon.svelte";
  import FrameworkBadge from "$lib/components/ports/FrameworkBadge.svelte";
  import PortBadge from "$lib/components/ports/PortBadge.svelte";
  import PortDetails from "$lib/components/ports/PortDetails.svelte";
  import type { PortItem } from "$lib/types/port";
  import { trackPortDetailsOpened } from "$lib/analytics";
  import { formatMemory } from "$lib/utils/format";
  import { isExposed, portSource } from "$lib/utils/ports.js";

  let { ports }: { ports: PortItem[] } = $props();
  let expanded = $state(false);
  let port = $derived(ports[0]);

  function toggleDetails(): void {
    expanded = !expanded;
    if (expanded)
      trackPortDetailsOpened({
        has_framework: port.framework ? 1 : 0,
        has_favicon: port.cachedFaviconPath || port.faviconUrl ? 1 : 0,
      });
  }
  let listeners = $derived([...new Map(ports.map((item) => [item.port, item])).values()]);
  let label = $derived(port.displayName ?? port.processName ?? "Unknown process");
  let exposed = $derived(listeners.some((listener) => isExposed(listener.address)));
  let source = $derived(portSource(port));
</script>

<li class={`border-b border-[var(--border-subtle)] last:border-b-0 ${expanded ? "bg-[var(--surface-muted)]" : ""}`}>
  <button
    type="button"
    class={`relative flex w-full items-center gap-3 px-3.5 py-3 text-left transition-colors duration-150 hover:bg-[var(--surface-muted)] ${expanded ? "before:absolute before:inset-y-2 before:left-0 before:w-0.5 before:rounded-r before:bg-[var(--primary)]" : ""}`}
    aria-expanded={expanded}
    aria-label={`${expanded ? "Collapse" : "Expand"} ${label} process details`}
    onclick={toggleDetails}
  >
    <Favicon {port} {label} />
    <div class="min-w-0 flex-1">
      <div class="flex min-w-0 items-center gap-1.5 overflow-hidden">
        <span class="size-1.5 shrink-0 rounded-full bg-[var(--success)]"></span>
        <p class="truncate text-[13px] font-semibold text-[var(--text-primary)]">{label}</p>
        <FrameworkBadge framework={port.framework} />
        {#if source}
          <span class="inline-flex shrink-0 items-center gap-1 rounded border border-[var(--border-strong)] bg-[var(--surface-muted)] px-1.5 py-0.5 text-[10px] font-medium text-[var(--text-secondary)]">
            {#if source === "docker"}
              <Container size={11} strokeWidth={1.8} aria-hidden="true" />Docker
            {:else}
              <Terminal size={11} strokeWidth={1.8} aria-hidden="true" />WSL
            {/if}
          </span>
        {/if}
        {#if exposed}
          <span
            class="inline-flex shrink-0 items-center gap-1 rounded border border-[color-mix(in_srgb,var(--warning)_35%,transparent)] bg-[var(--warning-soft)] px-1.5 py-0.5 text-[10px] font-semibold text-[var(--warning)]"
            title="Listening on a non-loopback address — reachable from other devices on your network"
          >
            <Globe size={11} strokeWidth={2} aria-hidden="true" />Exposed
          </span>
        {/if}
      </div>
      <div class="mt-1.5 flex min-w-0 items-center gap-2 overflow-hidden">
        {#each listeners.slice(0, 3) as listener}
          <PortBadge port={listener.port} protocol={listener.protocol} />
        {/each}
        {#if listeners.length > 3}
          <span class="text-[10px] font-medium text-[var(--text-muted)]">+{listeners.length - 3}</span>
        {/if}
      </div>
    </div>
    <div class="shrink-0 text-right text-[10px] text-[var(--text-muted)]">
      <p class="font-mono">PID {port.pid ?? "—"}</p>
      <p class={`mt-1 flex items-center justify-end gap-1 ${port.memoryMb != null && port.memoryMb >= 50 ? "text-[var(--warning)]" : ""}`}>
        <MemoryStick size={11} strokeWidth={1.7} aria-hidden="true" />
        {formatMemory(port.memoryMb)}
      </p>
    </div>
    <ChevronDown
      size={16}
      strokeWidth={1.8}
      class={`shrink-0 text-[var(--text-muted)] transition-transform duration-150 ${expanded ? "rotate-180" : ""}`}
    />
  </button>

  {#if expanded}
    <div transition:slide={{ duration: 150 }}>
      <PortDetails {ports} />
    </div>
  {/if}
</li>
