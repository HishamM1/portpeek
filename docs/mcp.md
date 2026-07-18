# PortPeek MCP server

`portpeek-mcp` exposes PortPeek's local port inspection to [MCP](https://modelcontextprotocol.io)
clients (Claude Desktop, Cursor, Claude Code, etc.), so an AI assistant can see
what's listening on your machine — and free a stuck port — through the same
scan and safety guards as the GUI and CLI, instead of guessing at shell
commands.

Windows only, like the rest of PortPeek.

## Transport

Standard MCP **stdio**: newline-delimited JSON-RPC 2.0 on stdin/stdout. The
client launches `portpeek-mcp` as a child process; there is no network socket,
no port, and no daemon. It exits when the client closes stdin.

## Tools

| Tool | Arguments | What it does |
| --- | --- | --- |
| `list_ports` | `includeSystem?` (bool), `includeUdp?` (bool) | Lists listeners with **summary** fields only: port, protocol, address, pid, process/display name, memory, uptime, `isSystemPort`. Protected system ports are hidden unless `includeSystem` is true. |
| `inspect_port` | `port` (1–65535, required) | Full detail for one port, **including** executable path, working directory, and launch command. |
| `free_port` | `port` (1–65535, required) | Stops the process(es) holding the port. Destructive. Returns the freed PIDs. |

### Safety model

- **Read/write split.** `list_ports` deliberately omits executable paths,
  working directories, and commands so a broad listing never leaks filesystem
  paths or process arguments. Those fields are returned only by `inspect_port`,
  which the client must ask for by port — "beyond what the user explicitly
  requests" stays out of the default listing.
- **No side effects on read.** Listing and inspecting never stop a process.
- **Protected processes are refused.** `free_port` reuses the same
  `is_system_port` classification (owner SID / kernel / `%SystemRoot%`) as the
  GUI and CLI; a system-owned port returns an error and is never terminated.
- **Errors are structured.** Tool-level problems (nothing listening, protected
  port, termination failure) come back as an MCP tool result with `isError:
  true` and a plain message, so the model can react rather than crashing the
  session. Malformed requests get a JSON-RPC error.

## Setup

Build (or grab from a release) the `portpeek-mcp.exe` binary:

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml --bin portpeek-mcp
# → src-tauri/target/release/portpeek-mcp.exe
```

Then point your MCP client at it. Example client config:

```json
{
  "mcpServers": {
    "portpeek": {
      "command": "C:\\path\\to\\portpeek-mcp.exe"
    }
  }
}
```

For Claude Code:

```bash
claude mcp add portpeek -- C:\path\to\portpeek-mcp.exe
```

## Quick manual check

Pipe a session in by hand to confirm it responds:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_ports","arguments":{}}}' \
  | portpeek-mcp.exe
```
