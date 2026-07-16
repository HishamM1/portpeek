import { writable } from "svelte/store";
import { detectVsCode } from "$lib/tauri/commands";

export const vsCodeAvailable = writable(false);

export async function loadEditorAvailability(): Promise<void> {
  try {
    vsCodeAvailable.set(await detectVsCode());
  } catch {
    vsCodeAvailable.set(false);
  }
}
