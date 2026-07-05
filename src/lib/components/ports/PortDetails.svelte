<script lang="ts">
  import Binary from "@lucide/svelte/icons/binary";
  import Check from "@lucide/svelte/icons/check";
  import Clipboard from "@lucide/svelte/icons/clipboard";
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import CodeIcon from "@lucide/svelte/icons/code";
  import Container from "@lucide/svelte/icons/container";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import MemoryStick from "@lucide/svelte/icons/memory-stick";
  import Radio from "@lucide/svelte/icons/radio";
  import TerminalSquare from "@lucide/svelte/icons/square-terminal";
  import Terminal from "@lucide/svelte/icons/terminal";
  import PortActions from "$lib/components/ports/PortActions.svelte";
  import PortBadge from "$lib/components/ports/PortBadge.svelte";
  import { vsCodeAvailable } from "$lib/stores/editor";
  import { copyText, openInEditor, openPath } from "$lib/tauri/commands";
  import type { PortItem } from "$lib/types/port";
  import { fileName, formatMemory, formatUptime } from "$lib/utils/format";
  import { isExposed, portSource } from "$lib/utils/ports.js";

  let { ports }: { ports: PortItem[] } = $props();
  let port = $derived(ports[0]);
  let listeners = $derived([...new Map(ports.map((item) => [item.port, item])).values()]);
  let source = $derived(portSource(port));

  let copied = $state<string | null>(null);
  function copy(key: string, value: string | null): void {
    if (!value) return;
    void copyText(value).then(() => {
      copied = key;
      setTimeout(() => {
        if (copied === key) copied = null;
      }, 1200);
    });
  }

  let openError = $state<string | null>(null);
  function reportOpenError(error: unknown): void {
    const message = String(error);
    openError = message;
    setTimeout(() => {
      if (openError === message) openError = null;
    }, 2500);
  }
  function openFolder(path: string | null): void {
    if (!path) return;
    void openPath(path).catch(reportOpenError);
  }
  function openEditor(path: string | null): void {
    if (!path) return;
    void openInEditor(path).catch(reportOpenError);
  }
</script>

<div class="space-y-3 px-3.5 pb-1 pt-1">
  <div>
    <div class="mb-2 flex items-center gap-1.5">
      <Radio size={13} strokeWidth={1.8} class="text-[var(--success)]" aria-hidden="true" />
      <h3 class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">Listening on</h3>
      <span class="text-[10px] font-semibold uppercase tracking-wider text-[var(--primary)]">
        {listeners.length} {listeners.length === 1 ? "port" : "ports"}
      </span>
    </div>
    <ul class="space-y-1.5">
      {#each listeners as listener}
        <li class="flex min-w-0 items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface)] px-2.5 py-2">
          <PortBadge port={listener.port} protocol={listener.protocol} />
          <span
            class={`min-w-0 flex-1 truncate font-mono text-[11px] ${isExposed(listener.address) ? "text-[var(--warning)]" : "text-[var(--text-muted)]"}`}
            title={isExposed(listener.address)
              ? "Reachable from other devices on your network"
              : "Only reachable from this machine"}
          >
            {listener.address} · {isExposed(listener.address) ? "exposed" : "local only"}
          </span>
          <PortActions port={listener} mode="listener" />
        </li>
      {/each}
    </ul>
  </div>

  <div class="grid grid-cols-2 gap-2">
    <div class="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-muted)] px-3 py-2.5">
      <div class="flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-muted)]">
        <MemoryStick size={12} strokeWidth={1.7} aria-hidden="true" /> Memory
      </div>
      <p class="mt-1.5 font-mono text-[16px] font-semibold tabular-nums text-[var(--text-primary)]">{formatMemory(port.memoryMb)}</p>
    </div>
    <div class="rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-muted)] px-3 py-2.5">
      <div class="flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-muted)]">
        <Clock3 size={12} strokeWidth={1.7} aria-hidden="true" /> Uptime
      </div>
      <p class="mt-1.5 font-mono text-[16px] font-semibold tabular-nums text-[var(--text-primary)]">{formatUptime(port.uptimeSeconds)}</p>
    </div>
  </div>

  {#snippet copyButton(key: string, value: string | null, label: string)}
    {#if value}
      <button
        type="button"
        aria-label={copied === key ? "Copied" : label}
        title={copied === key ? "Copied" : label}
        onclick={() => copy(key, value)}
        class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-md text-[var(--text-muted)] opacity-0 transition-all hover:bg-[var(--surface-muted)] hover:text-[var(--text-primary)] focus-visible:opacity-100 group-hover:opacity-100"
      >
        {#if copied === key}
          <Check size={13} strokeWidth={2} class="text-[var(--success)]" aria-hidden="true" />
        {:else}
          <Clipboard size={13} strokeWidth={1.8} aria-hidden="true" />
        {/if}
      </button>
    {/if}
  {/snippet}

  <dl class="space-y-2.5">
    {#if source}
      <div class="flex items-start gap-2.5">
        {#if source === "docker"}
          <Container size={13} strokeWidth={1.7} class="mt-0.5 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
        {:else}
          <Terminal size={13} strokeWidth={1.7} class="mt-0.5 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
        {/if}
        <div class="min-w-0 flex-1">
          <dt class="text-[11px] font-medium text-[var(--text-muted)]">Source</dt>
          <dd class="mt-0.5 text-[12px] text-[var(--text-primary)]">
            {source === "docker" ? "Docker" : "WSL"}
            <span class="text-[var(--text-muted)]"
              >— forwarded from a {source === "docker" ? "container" : "Linux distro"}; the real process runs inside it.</span
            >
          </dd>
        </div>
      </div>
    {/if}
    <div class="group flex items-start gap-2.5">
      <Binary size={13} strokeWidth={1.7} class="mt-0.5 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
      <div class="min-w-0 flex-1">
        <dt class="text-[11px] font-medium text-[var(--text-muted)]">Executable</dt>
        <dd class="mt-0.5 truncate font-mono text-[12px] text-[var(--text-primary)]" title={port.executablePath ?? ""}>{fileName(port.executablePath)}</dd>
      </div>
      {@render copyButton("exe", port.executablePath, "Copy executable path")}
    </div>
    <div class="group flex items-start gap-2.5">
      <FolderOpen size={13} strokeWidth={1.7} class="mt-0.5 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
      <div class="min-w-0 flex-1">
        <dt class="text-[11px] font-medium text-[var(--text-muted)]">Project</dt>
        <dd class="mt-0.5 truncate font-mono text-[12px] text-[var(--text-primary)]" title={port.workingDirectory ?? ""}>{port.workingDirectory ?? "—"}</dd>
      </div>
      {#if port.workingDirectory}
        <button
          type="button"
          aria-label="Open folder"
          title="Open folder"
          onclick={() => openFolder(port.workingDirectory)}
          class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-md text-[var(--text-muted)] opacity-0 transition-all hover:bg-[var(--surface-muted)] hover:text-[var(--text-primary)] focus-visible:opacity-100 group-hover:opacity-100"
        >
          <ExternalLink size={13} strokeWidth={1.8} aria-hidden="true" />
        </button>
        {#if $vsCodeAvailable}
          <button
            type="button"
            aria-label="Open in VS Code"
            title="Open in VS Code"
            onclick={() => openEditor(port.workingDirectory)}
            class="mt-0.5 grid size-6 shrink-0 place-items-center rounded-md text-[var(--text-muted)] opacity-0 transition-all hover:bg-[var(--surface-muted)] hover:text-[var(--text-primary)] focus-visible:opacity-100 group-hover:opacity-100"
          >
            <CodeIcon size={13} strokeWidth={1.8} aria-hidden="true" />
          </button>
        {/if}
      {/if}
      {@render copyButton("project", port.workingDirectory, "Copy project path")}
    </div>
    <div class="group flex items-start gap-2.5">
      <TerminalSquare size={13} strokeWidth={1.7} class="mt-0.5 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
      <div class="min-w-0 flex-1">
        <dt class="text-[11px] font-medium text-[var(--text-muted)]">Command</dt>
        <dd class="mt-0.5 break-all font-mono text-[12px] text-[var(--text-primary)]" title={port.command ?? ""}>{port.command ?? "—"}</dd>
      </div>
      {@render copyButton("command", port.command, "Copy command")}
    </div>
  </dl>

  {#if openError}
    <p class="text-[11px] text-[var(--danger)]" role="alert">{openError}</p>
  {/if}
</div>

<PortActions {port} mode="process" processPorts={listeners.map((l) => l.port)} />
