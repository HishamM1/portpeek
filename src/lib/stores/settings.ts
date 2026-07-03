import { writable } from "svelte/store";
import { getSettings, updateSettings } from "$lib/tauri/commands";
import type { Settings } from "$lib/types/settings";
import { DEFAULT_SETTINGS } from "$lib/utils/constants";

export const settings = writable<Settings>({ ...DEFAULT_SETTINGS });
export const settingsLoading = writable(false);
export const settingsError = writable<string | null>(null);
export const settingsLoaded = writable(false);

export async function loadSettings(): Promise<void> {
  settingsLoading.set(true);
  settingsError.set(null);
  try {
    const value = await getSettings();
    settings.set(value);
    applyTheme(value.theme);
  } catch (error) {
    settingsError.set(String(error));
  } finally {
    settingsLoading.set(false);
    settingsLoaded.set(true);
  }
}

export async function saveSettings(value: Settings): Promise<void> {
  settingsLoading.set(true);
  settingsError.set(null);
  try {
    const saved = await updateSettings(value);
    settings.set(saved);
    applyTheme(saved.theme);
  } catch (error) {
    settingsError.set(String(error));
  } finally {
    settingsLoading.set(false);
  }
}

function applyTheme(theme: Settings["theme"]): void {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme = theme;
}
