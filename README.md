<div align="center">
  <img src="static/portpeek-icon.svg" width="72" alt="PortPeek" />
  <h1>PortPeek</h1>
  <p>A small Windows tray app that shows what's listening on your local ports, and frees a busy one in a click.</p>
  <p><a href="https://hishamm1.github.io/portpeek/"><b>▶ Try the interactive demo</b></a></p>
</div>

## What it does

Every process listening on a local port, with its PID, memory, uptime, project folder, and the command that started it. When `:3000` is already taken, type the port, find what owns it, and stop it without touching `netstat` or Task Manager.

## Features

- **Full process context.** PID, memory, uptime, executable, working directory, and command.
- **Free a port in one click.** Search a port number and stop whatever holds it.
- **Exposure badge.** Flags anything bound to `0.0.0.0` and reachable on your network.
- **Framework aware.** Labels Postgres, Vite, Next.js, Docker, WSL, and others.
- **Filters and search.** Dev vs system ports, TCP/UDP, and live search over ports, names, and PIDs.
- **Native and small.** Built with Tauri, around 5 MB, runs from the tray.

## Install

Download the latest installer from [Releases](https://github.com/HishamM1/portpeek/releases/latest). Windows 10/11, x64.

Prefer a package manager? Install it with `winget install PortPeek`.

On first launch Windows may show a SmartScreen warning because the installer is not currently code-signed. This is a reputation warning, not a malware verdict: download only from the official GitHub release, then choose **More info**, then **Run anyway** if you trust the source.

## See it

[Try the interactive demo](https://hishamm1.github.io/portpeek/) — a playable PortPeek right in your browser.

## Build from source

Needs [Rust](https://rustup.rs), [Node](https://nodejs.org), and [pnpm](https://pnpm.io).

```bash
pnpm install
pnpm tauri dev      # run it
pnpm tauri build    # produce the installer
```

Frontend is Svelte 5 + Vite; the backend is Rust (Tauri v2). See [CONTRIBUTING.md](CONTRIBUTING.md) for a tour of the code.

## Privacy

PortPeek reads your ports **locally** — process names, paths, ports, and project details never leave your machine.

It sends **anonymous, opt-out usage analytics** (via [Aptabase](https://aptabase.com)) so we can prioritize what to build: counts and enums only — which features get used, whether scans succeed, roughly how many ports you run. **Never** ports, paths, PIDs, process names, URLs, or your search text. Turn it off anytime under **Settings › Privacy › Share anonymous usage**.

## License

MIT
