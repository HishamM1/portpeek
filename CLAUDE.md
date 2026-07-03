# CLAUDE.md

**Read [`AGENTS.md`](AGENTS.md) first** — it's the full project guide (overview, architecture, status, workflow, settled decisions). This file is just the fast orientation + gotchas so you don't re-explore.

## What this is
PortPeek: a **Windows tray utility** that shows what's listening on local ports (process, framework, project, quick actions). **Tauri v2 + Svelte 5 + Vite + Tailwind v4 + Rust.** **Not SvelteKit.** Windows-only for now.

## The 60-second mental model
- Frontend (`src/`) and Rust backend (`src-tauri/`) are separate, bridged by IPC. The frontend calls Rust via `invoke("cmd")` — all wrappers are in **`src/lib/tauri/commands.ts`** (start there).
- Backend wiring is **`src-tauri/src/app/setup.rs`** (plugins, `invoke_handler!`, tray). Scanning: `src-tauri/src/platform/windows/`. Detection: `src-tauri/src/domain/detection/`.
- Data flow: `refreshPorts()` → `invoke("list_ports")` → scan + enrich → `PortItem[]` → `ports` store → `filters.ts` (`scopedPorts`→`visiblePorts`) → `PortList`→`PortRow`→`PortDetails`.
- `PortItem`/`Settings` are **mirrored by hand** in Rust (`domain/*/types.rs`) and TS (`lib/types/`). Change one → change the other.

## Before you edit
- Read `commands.ts`, `setup.rs`, and the specific store/component involved.
- Check `git branch` + PRs: **`release/1.0.2` is open as PR #1 (unmerged)** with SID system-detection, removed minimize-on-blur, dropdown fix. Roadmap plans + empty branches live under `docs/plans/` on `roadmap/next`.

## Commands
`pnpm install` · `pnpm tauri dev` (run app) · `pnpm dev` (frontend only — backend cmds won't work) · `pnpm check` (TS, must be 0) · `cargo test --manifest-path src-tauri/Cargo.toml` · `pnpm tauri build` (needs `TAURI_SIGNING_PRIVATE_KEY` env — CI has it).

## Gotchas that have bitten this project
- **Don't push `main` directly.** Branch → PR → CI → merge. Also the local `gh` has multiple accounts — run `gh auth switch --hostname github.com --user HishamM1` before any push or you'll 403.
- **CSS:** global element resets must go in `@layer base` — unlayered CSS overrides Tailwind utilities (caused a font-size bug).
- **Counts:** always derive from `scopedPorts`/`visiblePorts`, never a raw store, or the header and list disagree.
- **Version bumps:** update `package.json` + `src-tauri/Cargo.toml` + `src-tauri/tauri.conf.json` together, then tag `vX.Y.Z`.
- **Rust changes** need `pnpm tauri dev` rebuild, not a hot reload.
- **Stubs are not wired:** `domain/ports/filters.rs`, `infrastructure/cache.rs`, `domain/detection/types.rs`, `domain/processes/*` are TODO placeholders.

## Settled — don't re-litigate
Svelte+Vite (not SvelteKit) · Tauri v2 · teal "Berth" design + Geist fonts (no purple/AI look) · borderless taskbar window, opens bottom-right, draggable header · **minimize-on-blur removed on purpose (looping bug) — do not re-add** · system ports classified by process identity (SID/kernel/`%SystemRoot%`), not port number · auto-update via tauri-plugin-updater.

## Working style here
Keep changes small and reviewable. Prefer simple/native/existing over new heavy deps (the owner values this — see the "ponytail" preference). Match existing patterns and the Berth design. Verify (`pnpm check` + `cargo test`) before a PR. Ask before anything needing the owner's secrets/accounts (signing cert, tokens, telemetry key) or irreversible actions.

## Fixmind
Per the global rule, after fixing a meaningful bug (logic/API/state), call the fixmind `save_lesson` MCP tool. Skip it for pure styling/renames/formatting.
