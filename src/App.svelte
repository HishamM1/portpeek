<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fly } from "svelte/transition";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import PopupFrame from "$lib/components/layout/PopupFrame.svelte";
  import PortList from "$lib/components/ports/PortList.svelte";
  import SettingsPanel from "$lib/components/settings/SettingsPanel.svelte";
  import Toolbar from "$lib/components/toolbar/Toolbar.svelte";
  import { loadSettings } from "$lib/stores/settings";

  let settingsOpen = $state(false);

  async function closeSettings() {
    settingsOpen = false;
    await tick();
    document.getElementById("settings-toggle")?.focus();
  }

  const swapMs =
    typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches
      ? 0
      : 180;

  onMount(async () => {
    await loadSettings();
  });
</script>

<AppShell>
  <PopupFrame>
    <Toolbar bind:settingsOpen />
    <div class="relative min-h-0 flex-1">
      {#if settingsOpen}
        <div class="absolute inset-0" in:fly={{ x: 16, duration: swapMs }} out:fly={{ x: 16, duration: swapMs }}>
          <SettingsPanel onclose={closeSettings} />
        </div>
      {:else}
        <div class="absolute inset-0" in:fly={{ x: -16, duration: swapMs }} out:fly={{ x: -16, duration: swapMs }}>
          <PortList />
        </div>
      {/if}
    </div>
  </PopupFrame>
</AppShell>
