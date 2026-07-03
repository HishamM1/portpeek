export type Theme = "system" | "light" | "dark";
export type OpenProtocol = "http" | "https";

export interface Settings {
  showSystemPorts: boolean;
  refreshIntervalMs: number;
  theme: Theme;
  launchAtStartup: boolean;
  confirmBeforeKill: boolean;
  defaultOpenProtocol: OpenProtocol;
  shareUsage: boolean;
}
