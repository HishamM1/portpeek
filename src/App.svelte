<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import PopupFrame from "$lib/components/layout/PopupFrame.svelte";
  import PortList from "$lib/components/ports/PortList.svelte";
  import SettingsPanel from "$lib/components/settings/SettingsPanel.svelte";
  import Toolbar from "$lib/components/toolbar/Toolbar.svelte";
  import { loadSettings } from "$lib/stores/settings";

  let settingsOpen = $state(false);

  onMount(() => {
    void loadSettings();
  });
</script>

<AppShell>
  <PopupFrame>
    <Toolbar bind:settingsOpen />
    {#if settingsOpen}
      <div class="min-h-0 flex-1" in:fly={{ x: 12, duration: 140 }} out:fade={{ duration: 90 }}>
        <SettingsPanel />
      </div>
    {:else}
      <div class="min-h-0 flex-1" in:fade={{ duration: 120 }} out:fade={{ duration: 80 }}>
        <PortList />
      </div>
    {/if}
  </PopupFrame>
</AppShell>
