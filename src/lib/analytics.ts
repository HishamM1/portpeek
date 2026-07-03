import { trackEvent } from "@aptabase/tauri";
import { get } from "svelte/store";
import { settings, settingsLoaded } from "$lib/stores/settings";

// Central analytics wrapper — import the typed helpers below, never call trackEvent directly.
// Event catalog and privacy rules: docs/analytics.md.

type Props = Record<string, string | number>;

function track(name: string, props?: Props): Promise<void> {
  if (!get(settingsLoaded) || !get(settings).shareUsage) return Promise.resolve();
  return trackEvent(name, props).catch(() => {});
}

export function lengthBucket(len: number): "0" | "1_3" | "4_10" | "10_plus" {
  if (len === 0) return "0";
  if (len <= 3) return "1_3";
  if (len <= 10) return "4_10";
  return "10_plus";
}

export function errorType(error: unknown): "permission_denied" | "not_found" | "unknown" {
  const text = String(error).toLowerCase();
  if (/denied|permission|access|elevat/.test(text)) return "permission_denied";
  if (/not found|no such|missing|unknown port/.test(text)) return "not_found";
  return "unknown";
}

type ScanTrigger = "initial_load" | "manual_refresh";
type Protocol = "http" | "https";

// Lifecycle
export const trackAppStarted = () => track("app_started");

export const trackScanCompleted = (p: {
  trigger: ScanTrigger;
  ports_count: number;
  dev_server_count: number;
  framework_count: number;
  duration_ms: number;
}) => track("ports_scan_completed", p);
export const trackScanFailed = (p: { trigger: ScanTrigger; error_type: string }) =>
  track("ports_scan_failed", p);

// Port interaction
export const trackPortDetailsOpened = (p: { has_framework: 0 | 1; has_favicon: 0 | 1 }) =>
  track("port_details_opened", p);
export const trackPortOpened = (p: {
  protocol: Protocol;
  has_framework: 0 | 1;
  has_favicon: 0 | 1;
}) => track("port_opened_in_browser", p);
export const trackPortUrlCopied = (p: { protocol: Protocol }) => track("port_url_copied", p);

// Process actions
export const trackKillRequested = () => track("process_kill_requested");
export const trackKillConfirmed = () => track("process_kill_confirmed");
export const trackKillCancelled = () => track("process_kill_cancelled");
export const trackKillSucceeded = (p: { port_count: number; has_framework: 0 | 1 }) =>
  track("process_kill_succeeded", p);
export const trackKillFailed = (p: { error_type: string }) => track("process_kill_failed", p);

// UI / UX
export const trackSettingsOpened = () => track("settings_opened");
export const trackSearchUsed = (p: { query_length_bucket: string }) => track("search_used", p);
export const trackFilterChanged = (p: { filter_type: "system_ports" | "udp"; enabled: 0 | 1 }) =>
  track("filter_changed", p);
export const trackEmptyStateSeen = (p: { reason: "no_ports" | "no_search_match" }) =>
  track("empty_state_seen", p);

// Settings
export const trackSettingChanged = (p: { setting_key: string; value: string | number }) =>
  track("setting_changed", p);
export const trackUpdateChecked = () => track("update_checked");
export const trackUpdateInstalled = () => track("update_installed");
