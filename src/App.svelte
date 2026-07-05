<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import PopupFrame from "$lib/components/layout/PopupFrame.svelte";
  import PortList from "$lib/components/ports/PortList.svelte";
  import SettingsPanel from "$lib/components/settings/SettingsPanel.svelte";
  import Toolbar from "$lib/components/toolbar/Toolbar.svelte";
  import { loadEditorAvailability } from "$lib/stores/editor";
  import { loadSettings } from "$lib/stores/settings";
  import { trackAppStarted, trackSettingsOpened } from "$lib/analytics";

  let settingsOpen = $state(false);

  $effect(() => {
    if (settingsOpen) trackSettingsOpened();
  });

  const swapMs =
    typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches
      ? 0
      : 180;

  onMount(async () => {
    void loadEditorAvailability();
    await loadSettings();
    trackAppStarted();
  });
</script>

<AppShell>
  <PopupFrame>
    <Toolbar bind:settingsOpen />
    <div class="relative min-h-0 flex-1">
      {#if settingsOpen}
        <div class="absolute inset-0" in:fly={{ x: 16, duration: swapMs }} out:fly={{ x: 16, duration: swapMs }}>
          <SettingsPanel />
        </div>
      {:else}
        <div class="absolute inset-0" in:fly={{ x: -16, duration: swapMs }} out:fly={{ x: -16, duration: swapMs }}>
          <PortList />
        </div>
      {/if}
    </div>
  </PopupFrame>
</AppShell>
