<script lang="ts">
  import { onMount } from "svelte";
  import PortResolver from "$lib/components/ports/PortResolver.svelte";
  import PortRow from "$lib/components/ports/PortRow.svelte";
  import EmptyState from "$lib/components/shared/EmptyState.svelte";
  import ErrorState from "$lib/components/shared/ErrorState.svelte";
  import LoadingState from "$lib/components/shared/LoadingState.svelte";
  import { query, visiblePorts } from "$lib/stores/filters";
  import { ports, portsError, portsLoading, refreshPorts } from "$lib/stores/ports";
  import { settings } from "$lib/stores/settings";
  import { groupPorts } from "$lib/utils/ports.js";

  let numericQuery = $derived(/^\d{1,5}$/.test($query.trim()));
  let groups = $derived(groupPorts($visiblePorts));

  onMount(() => {
    void refreshPorts("initial_load");
    let timer: ReturnType<typeof setInterval>;
    const unsubscribe = settings.subscribe((value) => {
      clearInterval(timer);
      timer = setInterval(() => void refreshPorts(), value.refreshIntervalMs);
    });
    return () => {
      clearInterval(timer);
      unsubscribe();
    };
  });
</script>

<section aria-label="Local ports" class="flex h-full min-h-0 flex-col">
  {#if $portsLoading && $ports.length === 0}
    <LoadingState />
  {:else if $portsError && $ports.length === 0}
    <ErrorState message={$portsError} onRetry={() => refreshPorts("manual_refresh")} />
  {:else}
    <PortResolver />
    {#if groups.length === 0}
      {#if !numericQuery}
        <EmptyState
          reason={$query ? "no_search_match" : "no_ports"}
          message={$query ? "No ports match your search." : "No development ports are listening."}
        />
      {/if}
    {:else}
      <ul class="min-h-0 flex-1 overflow-y-auto" aria-live="polite">
        {#each groups as group (group[0].pid ?? group[0].id)}
          <PortRow ports={group} />
        {/each}
      </ul>
      <footer class="shrink-0 border-t border-[var(--border-subtle)] px-3.5 py-2 text-[10px] text-[var(--text-muted)]">
        {groups.length} {groups.length === 1 ? "process" : "processes"}
      </footer>
    {/if}
  {/if}
</section>
