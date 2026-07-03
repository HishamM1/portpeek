import { writable } from "svelte/store";
import { trackScanCompleted, trackScanFailed, errorType } from "$lib/analytics";
import { listPorts } from "$lib/tauri/commands";
import type { PortItem } from "$lib/types/port";

export const ports = writable<PortItem[]>([]);
export const portsLoading = writable(false);
export const portsError = writable<string | null>(null);
export const portsLastUpdated = writable<number | null>(null);

let refreshing = false;

export async function refreshPorts(trigger?: "initial_load" | "manual_refresh"): Promise<void> {
  if (refreshing) return;
  refreshing = true;
  portsLoading.set(true);
  portsError.set(null);

  const startedAt = Date.now();
  try {
    const result = await listPorts();
    ports.set(result);
    portsLastUpdated.set(Date.now());
    if (trigger) {
      trackScanCompleted({
        trigger,
        ports_count: result.length,
        dev_server_count: result.filter((p) => !p.isSystemPort).length,
        framework_count: result.filter((p) => p.framework != null).length,
        duration_ms: Date.now() - startedAt,
      });
    }
  } catch (error) {
    ports.set([]);
    portsError.set(error instanceof Error ? error.message : String(error));
    if (trigger) trackScanFailed({ trigger, error_type: errorType(error) });
  } finally {
    portsLoading.set(false);
    refreshing = false;
  }
}
