export function formatMemory(memoryMb: number | null): string {
  if (memoryMb === null) return "—";
  return memoryMb < 10 ? `${memoryMb.toFixed(1)} MB` : `${Math.round(memoryMb)} MB`;
}

export function formatUptime(seconds: number | null): string {
  if (seconds === null) return "—";
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3_600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86_400) return `${Math.floor(seconds / 3_600)}h ${Math.floor((seconds % 3_600) / 60)}m`;
  return `${Math.floor(seconds / 86_400)}d ${Math.floor((seconds % 86_400) / 3_600)}h`;
}

export function fileName(path: string | null): string {
  if (!path) return "—";
  return path.split(/[\\/]/).pop() || path;
}
