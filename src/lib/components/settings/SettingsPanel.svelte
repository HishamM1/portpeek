<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import { getVersion } from "@tauri-apps/api/app";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { onMount } from "svelte";
  import { trackSettingChanged, trackUpdateChecked, trackUpdateInstalled } from "$lib/analytics";
  import { settings, settingsError, settingsLoading, saveSettings } from "$lib/stores/settings";
  import type { OpenProtocol, Settings, Theme } from "$lib/types/settings";

  let { onclose }: { onclose?: () => void } = $props();

  function onKeydown(event: KeyboardEvent): void {
    // Escape backs out to the port list, mirroring the Back button. Skip when a
    // native <select> is focused so Escape can cancel its open dropdown instead.
    if (event.key !== "Escape" || !onclose) return;
    if (event.target instanceof HTMLSelectElement) return;
    event.preventDefault();
    onclose();
  }

  function update<K extends keyof Settings>(key: K, value: Settings[K]): void {
    void saveSettings({ ...$settings, [key]: value });
    if (key !== "shareUsage") {
      trackSettingChanged({ setting_key: key, value: typeof value === "boolean" ? (value ? 1 : 0) : value });
    }
  }

  function value(event: Event): string {
    return (event.currentTarget as HTMLSelectElement).value;
  }

  let version = $state("");
  let updateState = $state<"idle" | "checking" | "uptodate" | "available" | "installing" | "error">(
    "idle",
  );
  let pending = $state<Update | null>(null);
  let updateError = $state("");

  let updateMessage = $derived(
    updateState === "checking"
      ? "Checking for updates…"
      : updateState === "uptodate"
        ? "You're on the latest version."
        : updateState === "available"
          ? `Version ${pending?.version} is available.`
          : updateState === "installing"
            ? "Downloading and installing…"
            : updateState === "error"
              ? updateError
              : "Check for a new version.",
  );

  onMount(async () => {
    try {
      version = await getVersion();
    } catch {
      /* not running in the desktop app */
    }
  });

  async function checkForUpdate(): Promise<void> {
    updateState = "checking";
    updateError = "";
    trackUpdateChecked();
    try {
      const found = await check();
      if (found) {
        pending = found;
        updateState = "available";
      } else {
        updateState = "uptodate";
      }
    } catch (error) {
      updateError = String(error);
      updateState = "error";
    }
  }

  async function installUpdate(): Promise<void> {
    if (!pending) return;
    updateState = "installing";
    try {
      await pending.downloadAndInstall();
      await trackUpdateInstalled();
      await relaunch();
    } catch (error) {
      updateError = String(error);
      updateState = "error";
    }
  }
</script>

{#snippet toggle(on: boolean, label: string, onToggle: () => void)}
  <button
    type="button"
    role="switch"
    aria-checked={on}
    aria-label={label}
    disabled={$settingsLoading}
    onclick={onToggle}
    class={`relative h-6 w-11 shrink-0 rounded-full transition-colors disabled:opacity-50 ${
      on ? "bg-[var(--primary)]" : "bg-[var(--border-strong)]"
    }`}
  >
    <span
      class={`absolute top-0.5 size-5 rounded-full bg-white shadow-sm transition-all ${
        on ? "left-[1.375rem]" : "left-0.5"
      }`}
    ></span>
  </button>
{/snippet}

{#snippet chevron()}
  <svg
    class="pointer-events-none absolute right-2 top-1/2 size-3 -translate-y-1/2 text-[var(--text-muted)]"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="m6 9 6 6 6-6" />
  </svg>
{/snippet}

<svelte:window onkeydown={onKeydown} />

<section class="h-full overflow-y-auto px-4 pb-4 pt-3" aria-label="Settings">
  {#if onclose}
    <div class="mb-3 flex items-center gap-2 px-1">
      <button
        type="button"
        onclick={onclose}
        aria-label="Back to ports"
        class="flex size-7 items-center justify-center rounded-md text-[var(--text-secondary)] transition-colors hover:bg-[var(--hover-muted)] hover:text-[var(--text-primary)]"
      >
        <ArrowLeft size={16} strokeWidth={1.8} />
      </button>
      <h2 class="text-[13px] font-semibold">Settings</h2>
    </div>
  {/if}
  <h2 class="px-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">Display</h2>
  <div class="mt-2 divide-y divide-[var(--border-subtle)] rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-muted)]">
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Theme</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Match Windows or choose a mode</p>
      </div>
      <div class="relative shrink-0">
        <select
          disabled={$settingsLoading}
          value={$settings.theme}
          onchange={(event) => update("theme", value(event) as Theme)}
          class="h-8 appearance-none rounded-md border border-[var(--border-strong)] bg-[var(--surface)] pl-2 pr-7 text-xs"
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
        {@render chevron()}
      </div>
    </div>
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Show system ports</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Include services and privileged ports</p>
      </div>
      {@render toggle($settings.showSystemPorts, "Show system ports", () =>
        update("showSystemPorts", !$settings.showSystemPorts),
      )}
    </div>
  </div>

  <h2 class="mt-5 px-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">Behavior</h2>
  <div class="mt-2 divide-y divide-[var(--border-subtle)] rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-muted)]">
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Refresh interval</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">How often PortPeek rescans</p>
      </div>
      <div class="relative shrink-0">
        <select
          disabled={$settingsLoading}
          value={$settings.refreshIntervalMs}
          onchange={(event) => update("refreshIntervalMs", Number(value(event)))}
          class="h-8 appearance-none rounded-md border border-[var(--border-strong)] bg-[var(--surface)] pl-2 pr-7 text-xs"
        >
          <option value={1000}>1 second</option>
          <option value={2000}>2 seconds</option>
          <option value={5000}>5 seconds</option>
          <option value={10000}>10 seconds</option>
        </select>
        {@render chevron()}
      </div>
    </div>
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Default protocol</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Used by Open and Copy URL</p>
      </div>
      <div class="relative shrink-0">
        <select
          disabled={$settingsLoading}
          value={$settings.defaultOpenProtocol}
          onchange={(event) => update("defaultOpenProtocol", value(event) as OpenProtocol)}
          class="h-8 appearance-none rounded-md border border-[var(--border-strong)] bg-[var(--surface)] pl-2 pr-7 text-xs"
        >
          <option value="http">HTTP</option>
          <option value="https">HTTPS</option>
        </select>
        {@render chevron()}
      </div>
    </div>
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Confirm before stopping</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Prevent accidental process termination</p>
      </div>
      {@render toggle($settings.confirmBeforeKill, "Confirm before stopping", () =>
        update("confirmBeforeKill", !$settings.confirmBeforeKill),
      )}
    </div>
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Launch at startup</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Start PortPeek with Windows</p>
      </div>
      {@render toggle($settings.launchAtStartup, "Launch at startup", () =>
        update("launchAtStartup", !$settings.launchAtStartup),
      )}
    </div>
  </div>

  <h2 class="mt-5 px-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">Privacy</h2>
  <div class="mt-2 rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-muted)]">
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Share anonymous usage</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Anonymous events only — never ports, paths, or process names</p>
      </div>
      {@render toggle($settings.shareUsage, "Share anonymous usage", () =>
        update("shareUsage", !$settings.shareUsage),
      )}
    </div>
  </div>

  <h2 class="mt-5 px-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">About</h2>
  <div class="mt-2 rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-muted)]">
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Updates</p>
        <p
          class={`mt-0.5 text-[11px] ${updateState === "error" ? "text-[var(--danger)]" : "text-[var(--text-secondary)]"}`}
        >
          {updateMessage}
        </p>
      </div>
      {#if updateState === "available"}
        <button
          type="button"
          onclick={installUpdate}
          class="h-8 shrink-0 rounded-md bg-[var(--primary)] px-3 text-xs font-semibold text-[var(--text-inverse)] transition-opacity hover:opacity-90"
        >
          Install &amp; restart
        </button>
      {:else}
        <button
          type="button"
          onclick={checkForUpdate}
          disabled={updateState === "checking" || updateState === "installing"}
          class="h-8 shrink-0 rounded-md border border-[var(--border-strong)] bg-[var(--surface)] px-3 text-xs font-semibold transition-colors hover:bg-[var(--surface-muted)] disabled:opacity-50"
        >
          Check for updates
        </button>
      {/if}
    </div>
  </div>

  {#if $settingsError}
    <p class="mt-4 rounded-md bg-[var(--danger-soft)] p-2 text-[11px] text-[var(--danger)]" role="alert">
      {$settingsError}
    </p>
  {/if}

  <p class="mt-6 text-center font-mono text-[10px] text-[var(--text-muted)]">
    PortPeek {version ? `v${version}` : ""} · Windows
  </p>
</section>
