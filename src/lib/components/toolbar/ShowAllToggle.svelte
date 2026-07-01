<script lang="ts">
  import { settings, saveSettings } from "$lib/stores/settings";

  function toggle(): void {
    void saveSettings({ ...$settings, showSystemPorts: !$settings.showSystemPorts });
  }
</script>

<button
  type="button"
  aria-pressed={$settings.showSystemPorts}
  title={$settings.showSystemPorts
    ? "Showing all ports — click for dev ports only"
    : "Showing dev ports — click to include system ports"}
  onclick={toggle}
  class={`flex h-8 shrink-0 items-center gap-1.5 rounded-md border px-2 text-[10px] font-medium transition-colors ${
    $settings.showSystemPorts
      ? "border-[var(--primary-border)] bg-[var(--primary-soft)] text-[var(--primary)]"
      : "border-[var(--border-subtle)] text-[var(--text-secondary)] hover:bg-[var(--surface-muted)]"
  }`}
>
  <span class="size-1.5 rounded-full bg-[var(--primary)]"></span>
  {$settings.showSystemPorts ? "All ports" : "Dev ports"}
</button>
