import type { Settings } from "$lib/types/settings";

export const APP_NAME = "PortPeek";

export const DEFAULT_SETTINGS: Settings = {
  showSystemPorts: false,
  refreshIntervalMs: 2_000,
  theme: "system",
  launchAtStartup: false,
  confirmBeforeKill: true,
  minimizeOnBlur: false,
  defaultOpenProtocol: "http",
};
