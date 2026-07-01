<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { query } from "$lib/stores/filters";
  import { ports, refreshPorts } from "$lib/stores/ports";
  import { settings } from "$lib/stores/settings";
  import { killProcess } from "$lib/tauri/commands";

  let portNum = $derived.by(() => {
    const t = $query.trim();
    return /^\d{1,5}$/.test(t) && Number(t) <= 65535 ? Number(t) : null;
  });
  let match = $derived(portNum === null ? null : ($ports.find((p) => p.port === portNum) ?? null));
  let name = $derived(match?.displayName ?? match?.processName ?? (match ? `PID ${match.pid}` : ""));

  let confirming = $state(false);
  let busy = $state(false);

  $effect(() => {
    portNum;
    confirming = false;
  });

  function free(): void {
    if (!match || match.pid === null) return;
    if ($settings.confirmBeforeKill && !confirming) {
      confirming = true;
      return;
    }
    confirming = false;
    busy = true;
    void killProcess(match.pid).finally(() => {
      busy = false;
      void refreshPorts();
    });
  }
</script>

{#if portNum !== null}
  {#if match}
    <div class="mx-3.5 mt-3 flex items-center gap-2.5 rounded-lg border border-[var(--border-subtle)] border-l-2 border-l-[var(--warning)] bg-[var(--surface-muted)] px-3 py-2.5">
      <span class="min-w-0 flex-1 text-[12px] leading-snug text-[var(--text-primary)]">
        <span class="font-mono font-semibold">:{portNum}</span> in use ·
        <span class="font-medium">{name}</span>
        {#if match.pid !== null}<span class="font-mono text-[var(--text-muted)]"> · PID {match.pid}</span>{/if}
      </span>
      {#if match.isSystemPort}
        <span class="shrink-0 text-[11px] font-medium text-[var(--text-muted)]">Protected</span>
      {:else if confirming}
        <button
          type="button"
          onclick={() => (confirming = false)}
          class="inline-flex h-7 shrink-0 items-center rounded-md px-2 text-[11px] font-semibold text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface)]"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={free}
          class="inline-flex h-7 shrink-0 items-center gap-1.5 rounded-md bg-[var(--danger)] px-2.5 text-[11px] font-semibold text-[var(--text-inverse)] shadow-sm transition-opacity hover:opacity-90"
        >
          <Trash2 size={13} strokeWidth={2} aria-hidden="true" /> Free :{portNum}
        </button>
      {:else}
        <button
          type="button"
          disabled={busy || match.pid === null}
          onclick={free}
          class="inline-flex h-7 shrink-0 items-center gap-1.5 rounded-md bg-[var(--danger)] px-3 text-[11px] font-semibold text-[var(--text-inverse)] shadow-sm transition-opacity hover:opacity-90 disabled:pointer-events-none disabled:opacity-40"
        >
          <Trash2 size={13} strokeWidth={2} aria-hidden="true" /> Free port
        </button>
      {/if}
    </div>
  {:else}
    <div class="mx-3.5 mt-3 flex items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-muted)] px-3 py-2.5">
      <Check size={15} strokeWidth={2} class="shrink-0 text-[var(--success)]" aria-hidden="true" />
      <span class="text-[12px] text-[var(--text-secondary)]">
        <span class="font-mono font-semibold text-[var(--text-primary)]">:{portNum}</span> is free — nothing is listening
      </span>
    </div>
  {/if}
{/if}
