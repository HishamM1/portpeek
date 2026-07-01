import { derived, writable } from "svelte/store";
import { ports } from "./ports";
import { settings } from "./settings";
import type { PortItem } from "$lib/types/port";

export const showUdp = writable(false);
export const query = writable("");

export const scopedPorts = derived(
  [ports, settings, showUdp],
  ([$ports, $settings, $showUdp]) =>
    $ports.filter((port) => {
      if (!$settings.showSystemPorts && port.isSystemPort) return false;
      if (!$showUdp && port.protocol === "udp") return false;
      return true;
    }),
);

export const visiblePorts = derived([scopedPorts, query], ([$scopedPorts, $query]) => {
  const needle = $query.trim().toLowerCase();
  if (!needle) return $scopedPorts;
  return $scopedPorts.filter((port: PortItem) =>
    [
      port.port.toString(),
      port.address,
      port.processName,
      port.displayName,
      port.framework?.name,
      port.pid?.toString(),
    ].some((value) => value?.toLowerCase().includes(needle)),
  );
});
