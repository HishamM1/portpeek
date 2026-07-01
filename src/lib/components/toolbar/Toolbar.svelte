<script lang="ts">
  import Activity from "@lucide/svelte/icons/activity";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Settings from "@lucide/svelte/icons/settings";
  import X from "@lucide/svelte/icons/x";
  import IconButton from "$lib/components/shared/IconButton.svelte";
  import ProtocolToggle from "$lib/components/toolbar/ProtocolToggle.svelte";
  import SearchBox from "$lib/components/toolbar/SearchBox.svelte";
  import ShowAllToggle from "$lib/components/toolbar/ShowAllToggle.svelte";
  import { visiblePorts } from "$lib/stores/filters";
  import { refreshPorts } from "$lib/stores/ports";
  import { hidePopupWindow } from "$lib/tauri/commands";

  let { settingsOpen = $bindable(false) }: { settingsOpen?: boolean } = $props();

  let refreshing = $state(false);
  async function manualRefresh() {
    if (refreshing) return;
    refreshing = true;
    await Promise.all([refreshPorts(), new Promise((resolve) => setTimeout(resolve, 600))]);
    refreshing = false;
  }
</script>

<header class="shrink-0 border-b border-[var(--border-subtle)] px-3.5 pb-3 pt-3">
  <div class="flex items-center gap-2.5">
    <div class="grid size-8 place-items-center rounded-lg bg-[var(--primary)] text-[var(--text-inverse)] shadow-sm">
      <Activity size={17} strokeWidth={2.2} />
    </div>
    <div class="min-w-0 flex-1">
      <h1 class="text-sm font-semibold leading-tight tracking-[-0.01em]">PortPeek</h1>
      <p class="mt-0.5 flex items-center gap-1.5 text-[11px] text-[var(--text-secondary)]" aria-live="polite">
        <span class="size-1.5 rounded-full bg-[var(--success)]"></span>
        {$visiblePorts.length} listening
      </p>
    </div>
    <IconButton label="Refresh ports" disabled={refreshing} onclick={() => void manualRefresh()}>
      <RefreshCw size={15} strokeWidth={1.8} class={refreshing ? "animate-spin" : ""} />
    </IconButton>
    <IconButton
      label={settingsOpen ? "Back to ports" : "Open settings"}
      active={settingsOpen}
      onclick={() => (settingsOpen = !settingsOpen)}
    >
      <Settings size={15} strokeWidth={1.8} />
    </IconButton>
    <IconButton label="Hide PortPeek" onclick={() => void hidePopupWindow()}>
      <X size={16} strokeWidth={1.8} />
    </IconButton>
  </div>

  {#if !settingsOpen}
    <div class="mt-3 flex gap-2">
      <SearchBox />
      <ShowAllToggle />
      <ProtocolToggle />
    </div>
  {/if}
</header>
