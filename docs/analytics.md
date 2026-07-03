# Analytics

PortPeek uses [Aptabase](https://aptabase.com) for **anonymous, opt-out** usage analytics.
All tracking goes through one wrapper — `src/lib/analytics.ts` (frontend) and
`src-tauri/src/app/analytics.rs` (the few Rust-side events). Never call `trackEvent` directly.

## Principles

- **Opt-out, on by default.** Gated live on `settings.shareUsage`; toggling it off stops events with no restart.
- **Off in contributor builds.** The Rust plugin only initializes when built with an `APTABASE_KEY`; without it, every `track()` is a silent no-op.
- **Best-effort.** A failed send never throws, blocks the UI, or logs noise.
- **Meaningful flows only** — not every click, render, or hover. We track: did scanning work, did the user find/open/copy/kill a port, are settings/search/filters used, are errors happening.
- **Debug/Release split** is automatic (Aptabase sets `isDebug` from the build profile), as is OS / app-version / locale enrichment — we never send those ourselves.

## Privacy rules (hard constraints)

Properties are **strings and numbers only**, and only from these shapes:

| Allowed | Never sent |
|---|---|
| counts (`ports_count`), durations (`duration_ms`) | ports, PIDs, addresses |
| buckets (`query_length_bucket`) | file paths, working dirs, executables |
| enums (`protocol`, `trigger`, `setting_key`) | project names, process names, command lines |
| booleans as `1`/`0` (`has_framework`) | full URLs, the search query text |
| coarse error codes | raw error / exception messages |

Errors are mapped to stable codes (`permission_denied`, `not_found`, `unknown`) — never the message.

## Event catalog

Legend: **✅ implemented** · **🔜 planned** (documented, not wired).

### Lifecycle
| Event | Props | Why | |
|---|---|---|---|
| `app_started` | — | DAU / retention / version split (Aptabase adds version+OS). | ✅ |
| `app_exited`, `app_started.source` (`window`/`tray`/`startup`) | — | Session shape + launch path. Needs Rust plumbing; low priority. | 🔜 |

### Port scanning (the core value)
Only `initial_load` and `manual_refresh` emit — **auto-refresh scans are silent** (they run every few seconds).
| Event | Props | Why | |
|---|---|---|---|
| `ports_scan_completed` | `trigger` (`initial_load`/`manual_refresh`), `ports_count`, `dev_server_count`, `framework_count`, `duration_ms` | Does scanning work, how many dev servers people run, is detection useful (aggregate — no per-item events), perf. | ✅ |
| `ports_scan_failed` | `trigger`, `error_type` | Reliability of the core feature. | ✅ |

### Port interaction
| Event | Props | Why | |
|---|---|---|---|
| `port_details_opened` | `has_framework` (`1`/`0`), `has_favicon` | Do users drill into a result. | ✅ |
| `port_opened_in_browser` | `protocol` (`http`/`https`), `has_framework`, `has_favicon` | The "open my dev server" payoff. | ✅ |
| `port_url_copied` | `protocol` | Copy-vs-open preference. | ✅ |

### Process actions (high intent)
| Event | Props | Why | |
|---|---|---|---|
| `process_kill_requested` | — | Intent to free a port (the activation funnel start). | ✅ |
| `process_kill_confirmed` / `process_kill_cancelled` | — | Confirm-dialog friction / accidental clicks. | ✅ |
| `process_kill_succeeded` | `port_count`, `has_framework` | **Activation** — the user freed a port. | ✅ |
| `process_kill_failed` | `error_type` | Kill reliability (e.g. permission denied). | ✅ |

### UI / UX
| Event | Props | Why | |
|---|---|---|---|
| `tray_opened` | `source` (`left_click`/`menu`) | Tray usage = ambient/daily-driver signal (Rust-side). | ✅ |
| `settings_opened` | — | Settings discovery. | ✅ |
| `search_used` | `query_length_bucket` (`0`/`1_3`/`4_10`/`10_plus`) | Is search used — **never the query itself**. | ✅ |
| `filter_changed` | `filter_type` (`system_ports`/`udp`), `enabled` (`1`/`0`) | Which filters matter. | ✅ |
| `empty_state_seen` | `reason` (`no_ports`/`no_search_match`) | Dead-end / friction moments. | ✅ |

### Settings
| Event | Props | Why | |
|---|---|---|---|
| `setting_changed` | `setting_key` (safe enum), `value` (enum/number, `1`/`0` for booleans) | Which settings people change. Values are never user-entered free text. | ✅ |
| `update_checked` / `update_installed` | — | Updater engagement + adoption. | ✅ |

### Rejected (intentionally not tracked)
- **`framework_detected` / `favicon_detected` per item** — detection runs on every scan for every port; per-item events would be a high-volume firehose and risk leaking project detail. The `framework_count` on `ports_scan_completed` answers "is detection working" at zero extra volume.
- **`tauri_command_failed` (generic)** — covered by the specific `*_failed` events; a catch-all invites raw-message leakage.

## Good vs bad properties

```ts
// GOOD
trackPortsScanCompleted({ trigger: "manual_refresh", ports_count: 12, dev_server_count: 3, framework_count: 2, duration_ms: 84 });
trackPortOpened({ protocol: "http", has_framework: 1, has_favicon: 0 });

// BAD — never do this
trackPortOpened({ url: "http://localhost:3000", project: "acme-web" });   // full URL + project name
trackSearchUsed({ query: "postgres" });                                    // raw query
trackScanFailed({ error: err.message });                                   // raw error message
```

## Setup still needed

- Set `APTABASE_KEY` (EU app key) as a build-time env var: locally before `pnpm tauri dev`/`build`, and as a GitHub Actions secret for release builds. It ships in the binary (not secret) but is never committed.
</content>
</invoke>
