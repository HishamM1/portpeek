<script lang="ts">
  import { settings, settingsError, settingsLoading, saveSettings } from "$lib/stores/settings";
  import type { OpenProtocol, Settings, Theme } from "$lib/types/settings";

  function update<K extends keyof Settings>(key: K, value: Settings[K]): void {
    void saveSettings({ ...$settings, [key]: value });
  }

  function value(event: Event): string {
    return (event.currentTarget as HTMLSelectElement).value;
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

<section class="h-full overflow-y-auto px-4 pb-4 pt-3" aria-label="Settings">
  <h2 class="px-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">Display</h2>
  <div class="mt-2 divide-y divide-[var(--border-subtle)] rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-muted)]">
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Theme</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Match Windows or choose a mode</p>
      </div>
      <select
        disabled={$settingsLoading}
        value={$settings.theme}
        onchange={(event) => update("theme", value(event) as Theme)}
        class="h-8 rounded-md border border-[var(--border-strong)] bg-[var(--surface)] px-2 text-xs"
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
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
      <select
        disabled={$settingsLoading}
        value={$settings.refreshIntervalMs}
        onchange={(event) => update("refreshIntervalMs", Number(value(event)))}
        class="h-8 rounded-md border border-[var(--border-strong)] bg-[var(--surface)] px-2 text-xs"
      >
        <option value={1000}>1 second</option>
        <option value={2000}>2 seconds</option>
        <option value={5000}>5 seconds</option>
        <option value={10000}>10 seconds</option>
      </select>
    </div>
    <div class="flex items-center justify-between gap-4 px-3.5 py-3">
      <div class="min-w-0">
        <p class="text-[13px] font-semibold">Default protocol</p>
        <p class="mt-0.5 text-[11px] text-[var(--text-secondary)]">Used by Open and Copy URL</p>
      </div>
      <select
        disabled={$settingsLoading}
        value={$settings.defaultOpenProtocol}
        onchange={(event) => update("defaultOpenProtocol", value(event) as OpenProtocol)}
        class="h-8 rounded-md border border-[var(--border-strong)] bg-[var(--surface)] px-2 text-xs"
      >
        <option value="http">HTTP</option>
        <option value="https">HTTPS</option>
      </select>
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

  {#if $settingsError}
    <p class="mt-4 rounded-md bg-[var(--danger-soft)] p-2 text-[11px] text-[var(--danger)]" role="alert">
      {$settingsError}
    </p>
  {/if}

  <p class="mt-6 text-center font-mono text-[10px] text-[var(--text-muted)]">PortPeek v1.0.0 · Windows</p>
</section>
