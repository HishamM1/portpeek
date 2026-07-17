import { invoke } from "@tauri-apps/api/core";
import type { PortItem } from "$lib/types/port";
import type { OpenProtocol, Settings } from "$lib/types/settings";

export const listPorts = () => invoke<PortItem[]>("list_ports");
export const killProcess = (pid: number) => invoke<void>("kill_process", { pid });
export const killProcessElevated = (pid: number) => invoke<void>("kill_process_elevated", { pid });
export const restartProcess = (pid: number, command: string, workingDirectory: string) =>
  invoke<void>("restart_process", { pid, command, workingDirectory });
export const openLocalhostUrl = (port: number, protocol: OpenProtocol) =>
  invoke<void>("open_localhost_url", { port, protocol });
export const copyLocalhostUrl = (port: number, protocol: OpenProtocol) =>
  invoke<void>("copy_localhost_url", { port, protocol });
export const copyPort = (port: number) => invoke<void>("copy_port", { port });
export const copyText = (text: string) => invoke<void>("copy_text", { text });
export const openPath = (path: string) => invoke<void>("open_path", { path });
export const openInEditor = (path: string) => invoke<void>("open_in_editor", { path });
export const detectVsCode = () => invoke<boolean>("detect_vscode");
export const getSettings = () => invoke<Settings>("get_settings");
export const updateSettings = (settings: Settings) =>
  invoke<Settings>("update_settings", { settings });
export const showPopupWindow = () => invoke<void>("show_popup_window");
export const hidePopupWindow = () => invoke<void>("hide_popup_window");
export const minimizePopupWindow = () => invoke<void>("minimize_popup_window");
