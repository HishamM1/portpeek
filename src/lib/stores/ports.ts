import { writable } from "svelte/store";
import { listPorts } from "$lib/tauri/commands";
import type { PortItem } from "$lib/types/port";

export const ports = writable<PortItem[]>([]);
export const portsLoading = writable(false);
export const portsError = writable<string | null>(null);
export const portsLastUpdated = writable<number | null>(null);

let refreshing = false;

export async function refreshPorts(_trigger?: "initial_load" | "manual_refresh"): Promise<void> {
  if (refreshing) return;
  refreshing = true;
  portsLoading.set(true);
  portsError.set(null);

  try {
    const result = await listPorts();
    ports.set(result);
    portsLastUpdated.set(Date.now());
  } catch (error) {
    ports.set([]);
    portsError.set(error instanceof Error ? error.message : String(error));
  } finally {
    portsLoading.set(false);
    refreshing = false;
  }
}
