<script lang="ts">
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Hash from "@lucide/svelte/icons/hash";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import IconButton from "$lib/components/shared/IconButton.svelte";
  import {
    errorType,
    trackKillCancelled,
    trackKillConfirmed,
    trackKillFailed,
    trackKillRequested,
    trackKillSucceeded,
    trackPortOpened,
    trackPortUrlCopied,
  } from "$lib/analytics";
  import { refreshPorts } from "$lib/stores/ports";
  import { settings } from "$lib/stores/settings";
  import {
    copyLocalhostUrl,
    copyPort,
    killProcess,
    openLocalhostUrl,
  } from "$lib/tauri/commands";
  import type { PortItem } from "$lib/types/port";

  let {
    port,
    mode = "listener",
    processPorts = [],
  }: {
    port: PortItem;
    mode?: "listener" | "process";
    processPorts?: number[];
  } = $props();
  let busy = $state<string | null>(null);
  let message = $state<string | null>(null);
  let confirming = $state(false);
  let label = $derived(port.displayName ?? port.processName ?? `PID ${port.pid}`);
  let portCount = $derived(processPorts.length || 1);
  let hasFramework = $derived<0 | 1>(port.framework ? 1 : 0);
  let hasFavicon = $derived<0 | 1>(port.cachedFaviconPath || port.faviconUrl ? 1 : 0);
  let endpointLabel = $derived(
    portCount > 1 ? processPorts.map((p) => `:${p}`).join("  ") : `localhost:${port.port}`,
  );

  async function run(action: string, operation: () => Promise<void>): Promise<void> {
    busy = action;
    message = null;
    try {
      await operation();
      if (action !== "open") message = action === "kill" ? "Process stopped" : "Copied";
      if (action === "open")
        trackPortOpened({
          protocol: $settings.defaultOpenProtocol,
          has_framework: hasFramework,
          has_favicon: hasFavicon,
        });
      else if (action === "url") trackPortUrlCopied({ protocol: $settings.defaultOpenProtocol });
      else if (action === "kill") {
        trackKillSucceeded({ port_count: portCount, has_framework: hasFramework });
        await refreshPorts();
      }
    } catch (error) {
      message = String(error);
      if (action === "kill") trackKillFailed({ error_type: errorType(error) });
    } finally {
      busy = null;
    }
  }

  function requestKill(): void {
    if (port.pid === null) return;
    trackKillRequested();
    if ($settings.confirmBeforeKill) {
      confirming = true;
      return;
    }
    void run("kill", () => killProcess(port.pid!));
  }

  function confirmKill(): void {
    confirming = false;
    trackKillConfirmed();
    void run("kill", () => killProcess(port.pid!));
  }

  function cancelKill(): void {
    confirming = false;
    trackKillCancelled();
  }

  function autofocus(node: HTMLElement): void {
    node.focus();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (confirming && event.key === "Escape") cancelKill();
  }}
/>

{#if mode === "listener"}
  <div class="flex shrink-0 items-center gap-0.5">
    <IconButton
      label={`Open localhost on port ${port.port}`}
      disabled={busy !== null}
      onclick={() => void run("open", () => openLocalhostUrl(port.port, $settings.defaultOpenProtocol))}
    >
      <ExternalLink size={14} strokeWidth={1.8} />
    </IconButton>
    <IconButton
      label="Copy localhost URL"
      disabled={busy !== null}
      onclick={() => void run("url", () => copyLocalhostUrl(port.port, $settings.defaultOpenProtocol))}
    >
      <Copy size={14} strokeWidth={1.8} />
    </IconButton>
    <IconButton
      label="Copy port number"
      disabled={busy !== null}
      onclick={() => void run("port", () => copyPort(port.port))}
    >
      <Hash size={14} strokeWidth={1.8} />
    </IconButton>
  </div>
{:else}
  <div class="flex min-h-12 items-center gap-2 border-t border-[var(--border-subtle)] px-3 py-2">
    {#if confirming}
      <span class="min-w-0 flex-1 text-[11px] font-medium leading-snug text-[var(--text-primary)]">
        Stop {label}?{#if portCount > 1}<span class="text-[var(--text-secondary)]"> Frees {portCount} ports.</span>{/if}
      </span>
      <button
        type="button"
        onclick={cancelKill}
        class="inline-flex h-8 shrink-0 items-center rounded-lg px-2.5 text-[11px] font-semibold text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-muted)]"
      >
        Cancel
      </button>
      <button
        type="button"
        use:autofocus
        onclick={confirmKill}
        class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-[var(--danger)] px-3 text-[11px] font-semibold text-[var(--text-inverse)] shadow-sm transition-opacity hover:opacity-90"
      >
        <Trash2 size={14} strokeWidth={2} aria-hidden="true" />
        Stop
      </button>
    {:else}
      <span class="min-w-0 flex-1 truncate font-mono text-[10px] text-[var(--text-muted)]" aria-live="polite">{message ?? endpointLabel}</span>
      <button
        type="button"
        disabled={port.pid === null || busy !== null || port.isSystemPort}
        title={port.isSystemPort
          ? "Protected system process"
          : portCount > 1
            ? `Stop process — frees ${portCount} ports`
            : "Stop process"}
        onclick={requestKill}
        class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-[var(--danger)] px-3 text-[11px] font-semibold text-[var(--text-inverse)] shadow-sm transition-opacity hover:opacity-90 disabled:pointer-events-none disabled:opacity-40"
      >
        <Trash2 size={14} strokeWidth={2} aria-hidden="true" />
        Stop process
      </button>
    {/if}
  </div>
{/if}
