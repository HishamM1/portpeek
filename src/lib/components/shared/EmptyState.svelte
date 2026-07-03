<script lang="ts">
  import Activity from "@lucide/svelte/icons/activity";
  import { onMount } from "svelte";
  import { trackEmptyStateSeen } from "$lib/analytics";

  let {
    message = "No listening TCP ports found.",
    reason,
  }: { message?: string; reason?: "no_ports" | "no_search_match" } = $props();

  onMount(() => {
    if (reason) trackEmptyStateSeen({ reason });
  });
</script>

<div class="grid flex-1 place-items-center px-8 text-center">
  <div>
    <div class="mx-auto grid size-10 place-items-center rounded-full bg-[var(--surface-muted)] text-[var(--text-muted)]">
      <Activity size={18} strokeWidth={1.6} />
    </div>
    <p class="mt-3 text-xs text-[var(--text-secondary)]">{message}</p>
  </div>
</div>
