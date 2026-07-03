# AGENTS.md — PortPeek

Cross-agent guide. Read this first; it should save you from re-discovering the project.

> **Keep this file current — it's the point.** When you do anything significant, update the matching section in the same change: ship a feature → add it to **Current features**; change architecture, add a command, add a dependency, or make a lasting call → update **Architecture** / **Settled decisions**; cut a release → bump **Versions & release tracking** (set the new shipped version, roll its scope into **Current features**, and record the next version + its planned features). Small tweaks don't need an entry; a "huge thing" always does.

---

## 1. Project overview

PortPeek is a **polished Windows desktop utility** that shows every process listening on a local port — so a developer can see "what's running on localhost" at a glance instead of reaching for `netstat`/Task Manager. It behaves like a **tray/menu-bar utility**: a small, borderless popup that lives in the system tray and opens bottom-right.

For each listener it shows: the **port(s)**, owning **process** (name, PID, memory, uptime), **executable**, **project folder**, launch **command**, a detected **framework** (with favicon where possible), a network **exposure** flag, and quick actions (open URL, copy URL/port, stop process, free a busy port).

**Stack:** Tauri v2 (Rust backend) + Svelte 5 (runes) + Vite 8 frontend + Tailwind CSS v4. **Windows-only today** (macOS/Linux planned). **Not SvelteKit** — plain Svelte + Vite (migrated away from SvelteKit).

Repo: `github.com/HishamM1/portpeek` (public, MIT © Hisham Medhat). Landing page: `hishamm1.github.io/portpeek/` (served from `docs/`).

## 2. Product goals & principles

- See local dev servers **without terminal commands**. The headline job is **resolving port conflicts** ("what's on :3000, free it").
- **Detect frameworks** (Next.js, Vite, Laravel, Rails, etc.) — implemented — and **project favicons** where possible — implemented (local project files).
- **Lightweight, native, not over-engineered.** ~5 MB, tray-resident, low memory. Prefer simple/native/maintainable over clever.
- **UX:** minimal, fast, polished; easy to understand at a glance. Prioritize developer convenience (open/copy/kill/refresh/identify). Avoid cluttered dashboards.
- **Design identity ("Berth"):** teal accent (`--primary`), petrol-dark surfaces, amber for warnings, coral reserved for destructive. Geist + Geist Mono fonts. Do **not** drift toward generic "AI" looks (no purple, no gradient headlines, real icons). See `app.css` tokens.

## 2b. Business view

- **Who:** developers on Windows who run local dev servers (web/app/db) and juggle multiple ports.
- **Core job (the wedge):** *"Something's on `:3000` — what is it, and give me the port back."* Port-conflict resolution is why people install. Secondary job: "what's listening on my machine right now?"
- **Value prop:** see local ports at a glance, **enriched** (process, framework, project, command) with **one-click actions** (open, copy, stop, free). vs. the alternatives: `netstat`/Task Manager (no context), TCPView (powerful but unpolished, not dev-aware), `npx kill-port` (CLI, one-shot). PortPeek's edge = **tray-resident + dev-aware + enriched + one-click + native/tiny (~5 MB, not Electron)**.
- **Retention reality:** today the app is **episodic** (opened in emergencies) — churn risk. The strategic lever is **ambient value** (watch/notify, pinned ports, at-a-glance tray) to become a daily driver. See planned items.
- **Growth loops:** a shareable CLI (`portpeek <port>`), winget/scoop presence, the interactive landing-page demo, OSS/GitHub stars, and auto-update keeping users current.
- **Business model:** **free & open source (MIT)** today. A possible future "Pro/Team" tier (remote/SSH monitoring, deep Docker/K8s integration, team dashboards) is *unconfirmed* — stay free-first; don't build paywalls without a decision.
- **Success metrics (once telemetry lands):** activation = *freed a port in week 1*; retention = DAU / return rate; plus per-feature usage. Until then, roadmap = judgment.
- **Distribution:** GitHub Releases (signed installer + auto-update), winget (first submission pending), landing page at `hishamm1.github.io/portpeek/`.

## 3. Architecture & patterns

**Two programs bridged by IPC.** The Svelte frontend never touches the OS; it calls Rust via `invoke("command_name")`. Learn the seam and the rest follows.

```
timer → refreshPorts() → invoke("list_ports") → Rust: scan (Win32) → enrich (process) →
enrich (framework/favicon) → PortItem[] → ports store → filters → PortList → PortRow → PortDetails
```

### Frontend (`src/`)
- Entry: `index.html` → `src/main.ts` (mounts Svelte, imports fonts + `app.css`) → `src/App.svelte` (`AppShell > PopupFrame > Toolbar` + `PortList`/`SettingsPanel`, crossfaded).
- `lib/components/` — `layout/` (AppShell, PopupFrame), `toolbar/` (Toolbar, SearchBox, ShowAllToggle, ProtocolToggle), `ports/` (PortList, PortRow, PortDetails, PortBadge, PortActions, PortResolver, Favicon, FrameworkBadge), `settings/` (SettingsPanel), `shared/` (Button, IconButton, EmptyState, ErrorState, LoadingState).
- `lib/stores/` — `ports.ts` (`ports`, `portsLoading`, `portsError`, `refreshPorts`), `settings.ts` (`settings`, load/save, `applyTheme`), `filters.ts` (`showUdp`, `query`, and derived `scopedPorts` → `visiblePorts` — **single source of truth** for both the header count and the list, so they never disagree).
- **`lib/tauri/commands.ts` — the IPC bridge.** Every Rust command has a thin `invoke` wrapper here. Read it first.
- `lib/types/` — `port.ts`, `settings.ts`, `framework.ts`. These **mirror the Rust structs by hand** (serde `camelCase`). Change one side → change the other.
- `lib/utils/` — `ports.js` (`groupPorts`, `brandSlug`, `isExposed`, `portSource` [docker/wsl heuristic], `isDatabase`; JSDoc-typed, has `ports.test.mjs`), `format.ts`, `constants.ts` (`DEFAULT_SETTINGS`).
- `app.css` — Tailwind v4 (`@import "tailwindcss"`), design tokens, light/dark/system theming via `[data-theme]`, `@theme` for fonts. ⚠️ **Global element resets go inside `@layer base`** — unlayered CSS beats `@layer utilities`, so an unlayered reset silently overrides Tailwind utilities (this caused a real font-size bug).

### Backend (`src-tauri/src/`) — layered
- `main.rs` → `lib.rs` (`portpeek_lib::run`) → **`app/setup.rs`** (the wiring: `tauri::Builder`, plugins, `invoke_handler!`, tray, startup-show). Read `setup.rs` first on the Rust side.
- `app/` — `setup.rs`, `tray.rs` (tray icon/menu; left-click → `window::toggle`), `window.rs` (`show`/`hide`/`toggle`, bottom-right positioning, `handle_event`).
- `commands/` — the `#[tauri::command]` handlers: `ports.rs` (`list_ports`, `kill_process`, `open_localhost_url`, `copy_localhost_url`, `copy_port`, `copy_text`), `settings.rs` (`get_settings`, `update_settings`), `window.rs` (`show_popup_window`, `hide_popup_window`). Register new commands in `setup.rs`'s `invoke_handler!`.
- `domain/` — pure logic, no OS calls. `ports/types.rs` (`PortItem`, `PortProtocol`), `settings/types.rs` (`Settings`, `Theme`, `OpenProtocol` + `validate`), `detection/` (`framework.rs` — package.json/command/config detection with confidence; `favicon.rs` — caches a project's favicon to the app cache dir; `project.rs` — `find_root` by markers; `mod.rs::enrich(app, items)` runs framework+favicon).
- `platform/windows/` — OS-specific. `ports.rs` (TCP+UDP enumeration via Win32 `GetExtendedTcp/UdpTable`), `processes.rs` (`enrich` via `sysinfo`, `is_system_process` classification, `terminate` with protections).
- `infrastructure/` — `logging.rs` (tracing), `paths.rs` (settings persistence: atomic write + backup).
- `state/app_state.rs` — `AppState { settings: Mutex<Settings> }`, managed via `app.manage(...)`.

**Where things live (for new work):**
- Port scanning / process detection → `platform/windows/` (add other OSes as `platform/<os>/`).
- Framework/favicon/project detection → `domain/detection/`.
- New backend capability → a `#[tauri::command]` in `commands/`, registered in `setup.rs`, wrapped in `commands.ts`, permission added to `capabilities/default.json` if needed.
- UI → `lib/components/…`; shared state → `lib/stores/`; helpers → `lib/utils/`; types → `lib/types/` (mirror Rust).

**Conventions:** Rust snake_case ↔ serde `camelCase` ↔ TS camelCase. Small focused Svelte components. Tailwind utilities + CSS-var tokens (no hardcoded colors — use `var(--…)`). Reuse `copy_text`, the `toggle` snippet, `IconButton`, etc. before adding new primitives.

## Current features (living list — **append every new feature here**)

What the product does today. Add a bullet whenever you ship user-facing behavior; note the version if handy.

- **Port list:** every TCP/UDP listener, grouped by process, refreshed on an interval.
- **Per-listener context:** port(s), PID, memory, uptime, executable, project folder, launch command.
- **Framework detection** (package.json / command / config file, with confidence) + framework badge.
- **Project favicon** caching from local files (`public/`, `static/`, `src/app/`).
- **Stop process** with protected-process guards and an optional confirm step.
- **"Free this port"** resolver — type a port number, see/stop whatever holds it.
- **Quick actions:** open localhost URL, copy URL, copy port, copy text (paths/command).
- **Filters & search:** dev vs system ports, UDP toggle, live search over ports/names/PIDs.
- **Exposure badge** when bound to `0.0.0.0`; **Docker/WSL** provenance chip (frontend heuristic).
- **Settings:** theme (system/light/dark), refresh interval, default protocol, confirm-before-kill, launch-at-startup — persisted to disk.
- **Window/tray:** tray icon + menu, borderless popup that opens bottom-right, lives in the taskbar, draggable by the header, single-instance; shows on manual launch, hidden on `--hidden` autostart.
- **Auto-update:** in-app "Check for updates" (download, verify, install, relaunch).
- **Around the app:** landing page with an interactive demo; CI (release + winget); auto-generated icons/tray.
- *(In flight, PR #1)* smarter **system-port classification by process identity**; settings **dropdown chevrons**.
- *(In flight, `feat/telemetry`)* **privacy-friendly usage analytics** — anonymous, **opt-out** (on by default), via Aptabase. Covers app lifecycle, port-scan (aggregate counts only), port/kill/settings/filter/search flows. **No PII, ports, paths, PIDs, process names, URLs, or query text.** "Share anonymous usage" toggle in Settings › Privacy. Full event catalog + privacy rules: **[`docs/analytics.md`](docs/analytics.md)**.

> Not everything above should be assumed bug-free — see status below for what's shipped vs in-flight and the known gaps.

## Versions & release tracking

The current/next-version tracker. **Keep it accurate on every release** — it's how the next session knows where things stand and what's slated next.

- **Shipped:** **v1.0.1** (on `main`).
- **Next / in flight:** **v1.0.2** — `release/1.0.2` (PR #1): SID-based system-port detection, removed minimize-on-blur, settings dropdown chevrons, `ci.yml`.
- **Planned after (unassigned to a version):** pull from Issues (`enhancement`) — #2 open-in-editor, #3 CLI, #4 code signing, #5 telemetry. Pin scope to a **GitHub milestone** when you schedule a version.

**On each release:**
1. Bump the version in all three files (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) and tag `vX.Y.Z`.
2. Update this section: set **Shipped** to the new version, move that version's scope into **Current features**, and fill **Next / in flight** with the next version + the features/issues planned for it.

## 4. Implementation status (honest)

**Shipped:** `main` = **v1.0.1** — the **Current features** list above is what's live (minus the "in flight" line).

**In flight — `release/1.0.2` branch, open as PR #1 (CI green, not merged):**
- **SID-based system-port classification** (owner account `S-1-5-18/19/20` / `pid<=4` / exe under `%SystemRoot%`), replacing the naive `port<1024` rule.
- **Removed "minimize when it loses focus"** — it caused a minimize loop on restore; do **not** re-add it.
- Settings dropdown chevrons; added `ci.yml` (check + cargo test on PRs). Version bumped to 1.0.2.

**Planned — tracked as GitHub Issues (label `enhancement`), each with a design plan in its body:**
- #2 open-in-editor & copy actions · #3 `portpeek` CLI companion · #4 Windows code signing (SmartScreen). *(#5 telemetry now in flight on `feat/telemetry`.)*
- Not yet filed: pin/group by project, watch & notify, restart process, real Docker/WSL mapping, macOS/Linux.
- **Ideas live in Issues, not branches.** Create a feature branch off `main` only when you actually start one (don't pre-create branches — they rot).

**Known gaps / risks / stubs:**
- **Installer is unsigned** → Windows SmartScreen "Unknown publisher". Biggest install-funnel issue.
- **Usage telemetry** built on `feat/telemetry` (Aptabase, opt-out) but **not merged**; no data until it ships and a signed build carries `APTABASE_KEY`. Contributor builds without that env var emit nothing.
- **Stub files** (placeholders, real logic lives elsewhere): `domain/ports/filters.rs`, `infrastructure/cache.rs`, `domain/detection/types.rs`, `domain/processes/*`. Don't assume they're wired.
- `FrameworkDetectionSource::HttpProbe` exists in the enum but HTTP probing isn't implemented.
- **winget:** first submission PR to `microsoft/winget-pkgs` is pending merge; the `winget.yml` auto-update PR only works after that lands.

## 5. Development workflow

Requires **Rust** (rustup), **Node 22+**, **pnpm** (`packageManager: pnpm@11.8.0`).

```bash
pnpm install
pnpm tauri dev        # run the full desktop app (backend commands work here)
pnpm dev              # frontend only in a browser — Rust commands will NOT work
pnpm tauri build      # produce the signed installer  (see signing note below)
pnpm check            # svelte-check (TypeScript) — must be 0 errors
pnpm test:ui          # node --test on src/lib/utils/ports.js
cargo test  --manifest-path src-tauri/Cargo.toml    # Rust unit tests
cargo build --manifest-path src-tauri/Cargo.toml    # compile-check (also validates capabilities)
```

- **`pnpm tauri build` requires update-signing env** (`bundle.createUpdaterArtifacts: true`): set `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, or let CI build it. `pnpm tauri dev` needs nothing.
- Build artifacts (`build/`, `src-tauri/target/`, `node_modules/`) are gitignored. Installers go to GitHub Releases, never committed.
- **Windows-only:** builds/tests assume a Windows toolchain (`windows-sys`). The scanning code is `#[cfg(target_os = "windows")]`; non-Windows returns an error.

**Release:** bump the version in **all three** files — `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` — then `git tag vX.Y.Z && git push --tags`. `release.yml` builds the signed installer + `latest.json` into a **draft** release; publish it manually. Publishing serves the updater endpoint (`releases/latest/download/latest.json`) and fires `winget.yml`.

## 6. Agent operating rules

**Before changing anything:** read `lib/tauri/commands.ts` (the seam), `app/setup.rs` (backend wiring), and the store/component you're touching. Check `git branch` and PR state — `release/1.0.2` (PR #1) is unmerged.

**Workflow:** work on a **branch** (`feat/…`, `fix/…`, `chore/…`), open a PR, let **CI** (`ci.yml`) pass, then merge → tag if releasing. **Do not push to `main` directly.** **Never add a `Co-Authored-By` (or any co-author) trailer to commits or PRs.**

**Keep changes small, safe, reviewable.** Don't rewrite large parts unnecessarily. Don't add heavy dependencies without a clear reason — prefer stdlib/native/existing deps. Always mirror a type change on both Rust and TS sides. Verify with `pnpm check` + `cargo test` before opening a PR. Rust changes require an app rebuild (`pnpm tauri dev`), not just a reload.

**Keep this guide updated in the same change.** Add shipped user-facing behavior to **Current features**; reflect big changes in **Architecture** / **Settled decisions** / **Implementation status**. Treat stale docs here as a bug.

**Ask for clarification** on: product/UX direction changes, anything requiring the owner's secrets/accounts (signing certs, tokens, Aptabase key), or destructive/irreversible actions.

**Settled decisions — do not re-litigate:**
- Plain **Svelte + Vite**, not SvelteKit. Entry is `index.html → main.ts → App.svelte`.
- **Tauri v2**, NSIS installer, `currentUser` install.
- **Teal "Berth"** visual identity; **Geist/Geist Mono** self-hosted fonts. No purple/generic-AI look.
- Window: **borderless, taskbar-present, draggable via header, opens bottom-right, shown on manual launch, hidden on `--hidden` autostart.** **Minimize-on-blur was removed on purpose (looping bug) — don't bring it back.**
- **System-port classification = process identity** (owner SID / kernel / `%SystemRoot%`), not port number.
- **Counts come from one source** (`scopedPorts`/`visiblePorts`) — never read a raw store for a count that a filtered list also shows.
- **Auto-update** via tauri-plugin-updater (minisign key; endpoint = GitHub `latest.json`).
- **Telemetry = Aptabase, opt-out (on by default), anonymous.** Key is injected via the `APTABASE_KEY` build-time env var (not committed — it ships in the binary anyway, so it's not secret); no key at build → plugin isn't initialized. **All tracking goes through the wrappers** (`src/lib/analytics.ts` frontend, `src-tauri/src/app/analytics.rs` for the few Rust events) — never call `trackEvent`/`track_event` directly. Every event gates on `settings.shareUsage`. Props are strings/numbers only; never PII, ports, paths, PIDs, process names, URLs, or query text. New event → add it to **[`docs/analytics.md`](docs/analytics.md)**.
- Global element CSS resets belong in **`@layer base`**.

## 7. Quick reference

- Frontend entry: `src/main.ts` · UI root: `src/App.svelte` · IPC: `src/lib/tauri/commands.ts`
- Backend entry: `src-tauri/src/app/setup.rs` · scanning: `src-tauri/src/platform/windows/` · detection: `src-tauri/src/domain/detection/`
- Design tokens: `src/app.css` · settings model: `src-tauri/src/domain/settings/types.rs` ↔ `src/lib/types/settings.ts`
- Version files (keep in sync): `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Contributor setup + code tour: `CONTRIBUTING.md` · roadmap: **GitHub Issues** (label `enhancement`)
