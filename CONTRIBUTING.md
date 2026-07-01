# Contributing to PortPeek

Thanks for helping out. PortPeek is a Windows tray app built with Tauri — a Svelte 5 + Vite frontend and a Rust backend.

## Prerequisites

- [Rust](https://rustup.rs)
- [Node](https://nodejs.org) 20+
- [pnpm](https://pnpm.io)

## Setup

```bash
pnpm install
pnpm tauri dev      # run the app
```

## How the code fits together

- `src/` — Svelte frontend: UI, stores, and the IPC bridge in `src/lib/tauri/commands.ts`.
- `src-tauri/` — Rust backend: `commands/` (the `#[tauri::command]` handlers), `platform/windows/` (port scanning and process control), `domain/` (types and detection).
- The frontend never touches the OS directly — it calls Rust via `invoke("command_name")`. Start reading at `commands.ts`, then follow a name into `src-tauri/src/app/setup.rs` (the handler list).

Adding a backend command: write it in `src-tauri/src/commands/`, register it in `src-tauri/src/app/setup.rs`, and add the matching wrapper in `src/lib/tauri/commands.ts`. The Rust `PortItem` and the TypeScript `PortItem` are mirrored by hand — change one, change the other.

## Before opening a PR

- `pnpm check` passes (0 errors).
- `cargo test --manifest-path src-tauri/Cargo.toml` passes.
- You ran `pnpm tauri dev` and confirmed the change works.
- Keep the PR focused on one thing, and say what it does and why.

## Reporting bugs or ideas

Open an issue using the templates. For bugs, include your Windows version and PortPeek version.
