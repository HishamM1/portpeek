import type { FrameworkDetection } from "./framework";

export type PortProtocol = "tcp" | "udp";

export interface PortItem {
  id: string;
  port: number;
  address: string;
  protocol: PortProtocol;
  pid: number | null;
  processName: string | null;
  displayName: string | null;
  memoryMb: number | null;
  uptimeSeconds: number | null;
  command: string | null;
  executablePath: string | null;
  workingDirectory: string | null;
  url: string | null;
  faviconUrl: string | null;
  cachedFaviconPath: string | null;
  framework: FrameworkDetection | null;
  isSystemPort: boolean;
}
