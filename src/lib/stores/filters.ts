import { derived, writable } from "svelte/store";
import { ports } from "./ports";
import { settings } from "./settings";
import { filterPorts } from "$lib/utils/ports.js";

export const showUdp = writable(false);
export const query = writable("");

export const scopedPorts = derived(
  [ports, settings, showUdp],
  ([$ports, $settings, $showUdp]) =>
    filterPorts($ports, $settings.showSystemPorts, $showUdp, "", $settings.pinnedPorts),
);

export const visiblePorts = derived(
  [ports, settings, showUdp, query],
  ([$ports, $settings, $showUdp, $query]) =>
    filterPorts($ports, $settings.showSystemPorts, $showUdp, $query, $settings.pinnedPorts),
);
