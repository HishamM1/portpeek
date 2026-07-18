//! PortPeek MCP server — exposes local listener inspection to MCP clients over
//! stdio (newline-delimited JSON-RPC 2.0). Reuses the same Win32 scan,
//! system-port classification, and termination guards as the GUI and CLI.
//!
//! ponytail: hand-rolled JSON-RPC over stdio instead of pulling in an async MCP
//! SDK + runtime. The stdio transport is one JSON object per line and the tool
//! surface is three calls — a full SDK would be more dependency than protocol.
//! Swap in `rmcp` only if we need streaming/HTTP transports or richer capabilities.

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use portpeek_lib::domain::ports::types::{PortItem, PortProtocol};

/// MCP protocol revisions this server supports. We echo the client's requested
/// version when it is one of these, otherwise we answer with our latest — as
/// the lifecycle spec requires. The first entry is our preferred/default.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut server = Server::default();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => server.handle(&request),
            Err(_) => Some(error(Value::Null, -32700, "parse error")),
        };

        if let Some(response) = response {
            write_message(&mut out, &response);
        }
    }
}

fn write_message(out: &mut impl Write, message: &Value) {
    // A dead stdout means the client is gone; nothing left to do but exit.
    if writeln!(out, "{message}").is_err() || out.flush().is_err() {
        std::process::exit(0);
    }
}

#[derive(Default)]
struct Server {
    /// Set when a valid `initialize` request has been answered.
    initialize_requested: bool,
    /// Set once the client sends `notifications/initialized` *after* initialize.
    /// Tool calls are refused until then, per the MCP lifecycle.
    initialized: bool,
}

impl Server {
    /// Routes a JSON-RPC message. Returns `Some(response)` for requests and
    /// `None` for notifications (no `id`) and unanswerable garbage.
    fn handle(&mut self, request: &Value) -> Option<Value> {
        let jsonrpc_ok = request.get("jsonrpc").and_then(Value::as_str) == Some("2.0");
        let method = request.get("method").and_then(Value::as_str);

        // A message with no `id` member is a notification: no reply, ever.
        let Some(raw_id) = request.get("id") else {
            if jsonrpc_ok
                && method == Some("notifications/initialized")
                && self.initialize_requested
            {
                self.initialized = true;
            }
            return None;
        };

        // From here it is a request and MUST get exactly one response. A request
        // id must be a string or an integer (not null, bool, float, or object).
        if !is_valid_id(raw_id) {
            return Some(error(
                Value::Null,
                -32600,
                "invalid request: id must be a string or integer",
            ));
        }
        let id = raw_id.clone();

        if !jsonrpc_ok {
            return Some(error(
                id,
                -32600,
                "invalid request: jsonrpc must be \"2.0\"",
            ));
        }
        let Some(method) = method else {
            return Some(error(id, -32600, "invalid request: missing method"));
        };
        // Params, when present, must be structured (object) — MCP uses objects.
        match request.get("params") {
            None | Some(Value::Null) | Some(Value::Object(_)) => {}
            Some(_) => {
                return Some(error(
                    id,
                    -32600,
                    "invalid request: params must be an object",
                ))
            }
        }
        let params = request.get("params").cloned().unwrap_or(Value::Null);

        match method {
            // `initialize` and `ping` are allowed before the handshake completes.
            "initialize" => {
                self.initialize_requested = true;
                Some(success(id, self.initialize_result(&params)))
            }
            "ping" => Some(success(id, json!({}))),
            "tools/list" | "tools/call" if !self.initialized => {
                Some(error(id, -32002, "server not initialized"))
            }
            "tools/list" => Some(success(id, json!({ "tools": tool_definitions() }))),
            "tools/call" => Some(call_tool(id, &params)),
            _ => Some(error(id, -32601, &format!("method not found: {method}"))),
        }
    }

    fn initialize_result(&self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        let protocol_version = negotiate_protocol_version(requested);

        json!({
            "protocolVersion": protocol_version,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "portpeek-mcp",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "instructions": "Inspect local listening ports. Use list_ports for an overview, \
        inspect_port for full detail on one port, and free_port to stop the process holding a \
        port. Protected system ports cannot be freed.",
        })
    }
}

/// A JSON-RPC 2.0 request id must be a string or integer — never null, bool,
/// fractional number, or a structured value.
fn is_valid_id(id: &Value) -> bool {
    id.is_string() || id.is_i64() || id.is_u64()
}

/// Echoes the client's requested version when we support it, else falls back to
/// our preferred version (the first supported entry).
fn negotiate_protocol_version(requested: Option<&str>) -> &str {
    match requested {
        Some(version) if SUPPORTED_PROTOCOL_VERSIONS.contains(&version) => version,
        _ => SUPPORTED_PROTOCOL_VERSIONS[0],
    }
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_ports",
            "description": "List processes listening on local ports (summary fields only). \
    Excludes protected system ports unless includeSystem is true.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "includeSystem": {
                        "type": "boolean",
                        "description": "Include OS/protected system ports. Default false.",
                    },
                    "includeUdp": {
                        "type": "boolean",
                        "description": "Include UDP listeners as well as TCP. Default false.",
                    },
                },
            },
        },
        {
            "name": "inspect_port",
            "description": "Inspect the process(es) listening on a specific port, including \
    executable path, working directory, and launch command.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 65535,
                        "description": "The local port to inspect.",
                    },
                },
                "required": ["port"],
            },
        },
        {
            "name": "free_port",
            "description": "Stop the process(es) holding a port. Refuses protected system \
    ports. This is destructive and terminates the owning process.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 65535,
                        "description": "The local port to free.",
                    },
                },
                "required": ["port"],
            },
        },
    ])
}

fn call_tool(id: Value, params: &Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
    if !arguments.is_null() && !arguments.is_object() {
        return error(
            id,
            -32602,
            "invalid params: \"arguments\" must be an object",
        );
    }

    let result = match name {
        "list_ports" => list_ports(&arguments),
        "inspect_port" => inspect_port(&arguments),
        "free_port" => free_port(&arguments),
        other => return error(id, -32602, &format!("unknown tool: {other}")),
    };

    match result {
        Ok(payload) => success(id, tool_result(&payload, false)),
        // Tool-level failures (protected port, nothing listening, termination
        // error) are reported as tool results with isError, not JSON-RPC errors,
        // so the model sees the message and can react.
        Err(message) => success(id, tool_result(&Value::String(message), true)),
    }
}

fn tool_result(payload: &Value, is_error: bool) -> Value {
    let text = match payload {
        Value::String(message) => message.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    };
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error,
    })
}

fn list_ports(arguments: &Value) -> Result<Value, String> {
    let include_system = optional_bool(arguments, "includeSystem")?.unwrap_or(false);
    let include_udp = optional_bool(arguments, "includeUdp")?.unwrap_or(false);

    let mut items = scan(include_udp)?;
    if !include_system {
        items.retain(|item| !item.is_system_port);
    }

    Ok(Value::Array(items.iter().map(summarize).collect()))
}

fn inspect_port(arguments: &Value) -> Result<Value, String> {
    let port = parse_port(arguments)?;
    let items = matching(port)?;
    if items.is_empty() {
        return Err(format!("nothing is listening on port {port}"));
    }
    Ok(Value::Array(items.iter().map(detail).collect()))
}

fn free_port(arguments: &Value) -> Result<Value, String> {
    let port = parse_port(arguments)?;
    // ponytail: authorize-then-terminate by PID, identical to the GUI and CLI
    // (the issue asks for the same guards). Windows offers no atomic
    // kill-by-port, so a scan→kill window exists where a PID could in principle
    // be reused; terminate() re-checks the protected-process list before
    // killing. The right fix is shared termination hardening across GUI/CLI/MCP,
    // not an MCP-only divergence — tracked separately, not in scope here.
    let items = matching(port)?;
    if items.is_empty() {
        return Err(format!("nothing is listening on port {port}"));
    }
    if items.iter().any(|item| item.is_system_port) {
        return Err(format!(
            "port {port} is owned by a protected system process; refusing to stop it"
        ));
    }

    let mut pids: Vec<u32> = items.iter().filter_map(|item| item.pid).collect();
    pids.sort_unstable();
    pids.dedup();
    if pids.is_empty() {
        return Err(format!("no owning process found for port {port}"));
    }

    let mut freed = Vec::new();
    let mut errors = Vec::new();
    for pid in pids {
        match terminate(pid) {
            Ok(()) => freed.push(pid),
            Err(error) => errors.push(format!("pid {pid}: {error}")),
        }
    }

    if freed.is_empty() {
        return Err(format!("failed to free port {port}: {}", errors.join("; ")));
    }

    Ok(json!({
        "port": port,
        "freedPids": freed,
        "errors": errors,
    }))
}

/// Summary projection for `list_ports` — deliberately omits executable path,
/// working directory, and command so a bare listing does not leak filesystem
/// paths or process arguments. Full detail is available via `inspect_port`.
fn summarize(item: &PortItem) -> Value {
    json!({
        "port": item.port,
        "protocol": protocol_label(item.protocol),
        "address": item.address,
        "pid": item.pid,
        "processName": item.process_name,
        "displayName": item.display_name,
        "memoryMb": item.memory_mb,
        "uptimeSeconds": item.uptime_seconds,
        "isSystemPort": item.is_system_port,
    })
}

/// Full projection for `inspect_port` — the caller has explicitly asked about
/// this port, so the path/command fields are included.
fn detail(item: &PortItem) -> Value {
    json!({
        "port": item.port,
        "protocol": protocol_label(item.protocol),
        "address": item.address,
        "pid": item.pid,
        "processName": item.process_name,
        "displayName": item.display_name,
        "memoryMb": item.memory_mb,
        "uptimeSeconds": item.uptime_seconds,
        "executablePath": item.executable_path,
        "workingDirectory": item.working_directory,
        "command": item.command,
        "url": item.url,
        "isSystemPort": item.is_system_port,
    })
}

/// Reads an optional boolean argument. Absent is fine (falls back to a default);
/// a present value of any non-boolean type — including explicit `null` — is
/// rejected rather than silently coerced (an MCP client is untrusted input, so
/// we hold it to the advertised schema).
fn optional_bool(arguments: &Value, key: &str) -> Result<Option<bool>, String> {
    match arguments.get(key) {
        None => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("\"{key}\" must be a boolean")),
    }
}

fn parse_port(arguments: &Value) -> Result<u16, String> {
    let port = arguments
        .get("port")
        .and_then(Value::as_u64)
        .ok_or("missing or invalid \"port\" (expected an integer 1-65535)")?;
    u16::try_from(port)
        .ok()
        .filter(|&port| port != 0)
        .ok_or_else(|| format!("port {port} is out of range (expected 1-65535)"))
}

fn matching(port: u16) -> Result<Vec<PortItem>, String> {
    let items = scan(true)?;
    Ok(items.into_iter().filter(|item| item.port == port).collect())
}

#[cfg(target_os = "windows")]
fn scan(include_udp: bool) -> Result<Vec<PortItem>, String> {
    let mut items = portpeek_lib::platform::windows::ports::list_tcp_listeners()
        .map_err(|error| format!("failed to list TCP listeners: {error}"))?;
    if include_udp {
        let mut udp = portpeek_lib::platform::windows::ports::list_udp_listeners()
            .map_err(|error| format!("failed to list UDP listeners: {error}"))?;
        items.append(&mut udp);
    }
    items.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| left.address.cmp(&right.address))
            .then_with(|| left.pid.cmp(&right.pid))
    });
    Ok(items)
}

#[cfg(not(target_os = "windows"))]
fn scan(_include_udp: bool) -> Result<Vec<PortItem>, String> {
    Err("port discovery is currently supported on Windows only".into())
}

#[cfg(target_os = "windows")]
fn terminate(pid: u32) -> Result<(), String> {
    portpeek_lib::platform::windows::processes::terminate(pid)
}

#[cfg(not(target_os = "windows"))]
fn terminate(_pid: u32) -> Result<(), String> {
    Err("process termination is currently supported on Windows only".into())
}

fn protocol_label(protocol: PortProtocol) -> &'static str {
    match protocol {
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    }
}

fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(port: u16, is_system: bool) -> PortItem {
        PortItem {
            id: format!("tcp-{port}"),
            port,
            address: "127.0.0.1".into(),
            protocol: PortProtocol::Tcp,
            pid: Some(1234),
            process_name: Some("node".into()),
            display_name: Some("node".into()),
            memory_mb: Some(42.0),
            uptime_seconds: Some(120),
            command: Some("node server.js --secret".into()),
            executable_path: Some("C:/proj/node.exe".into()),
            working_directory: Some("C:/proj".into()),
            url: Some("http://localhost:3000".into()),
            favicon_url: None,
            cached_favicon_path: None,
            framework: None,
            is_system_port: is_system,
        }
    }

    /// A server that has completed the initialize handshake.
    fn initialized_server() -> Server {
        let mut server = Server::default();
        server.handle(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
        }));
        server.handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        server
    }

    #[test]
    fn initialize_echoes_supported_version_and_falls_back_otherwise() {
        let mut server = Server::default();
        let supported = server
            .handle(&json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": { "protocolVersion": "2024-11-05" }
            }))
            .unwrap();
        assert_eq!(supported["result"]["protocolVersion"], "2024-11-05");
        assert!(supported["result"]["capabilities"]["tools"].is_object());

        let unsupported = server
            .handle(&json!({
                "jsonrpc": "2.0", "id": 2, "method": "initialize",
                "params": { "protocolVersion": "1.0.0" }
            }))
            .unwrap();
        assert_eq!(
            unsupported["result"]["protocolVersion"],
            SUPPORTED_PROTOCOL_VERSIONS[0]
        );
    }

    #[test]
    fn notifications_get_no_response() {
        let mut server = Server::default();
        let response = server.handle(&json!({
            "jsonrpc": "2.0", "method": "notifications/initialized"
        }));
        assert!(response.is_none());
    }

    #[test]
    fn unknown_method_returns_error() {
        let mut server = initialized_server();
        let response = server
            .handle(&json!({ "jsonrpc": "2.0", "id": 9, "method": "does/not/exist" }))
            .unwrap();
        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn tools_require_initialization_first() {
        let mut server = Server::default();
        let response = server
            .handle(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
            .unwrap();
        assert_eq!(response["error"]["code"], -32002);
    }

    #[test]
    fn request_without_method_is_invalid_request() {
        let mut server = Server::default();
        let response = server
            .handle(&json!({ "jsonrpc": "2.0", "id": 5 }))
            .unwrap();
        assert_eq!(response["error"]["code"], -32600);
    }

    #[test]
    fn rejects_bad_jsonrpc_version_and_id_type() {
        let mut server = Server::default();
        // Wrong jsonrpc version.
        let bad_version = server
            .handle(&json!({ "jsonrpc": "1.0", "id": 1, "method": "ping" }))
            .unwrap();
        assert_eq!(bad_version["error"]["code"], -32600);
        // Non-string/integer id (float, bool, object) is invalid; reply id null.
        let bad_id = server
            .handle(&json!({ "jsonrpc": "2.0", "id": 1.5, "method": "ping" }))
            .unwrap();
        assert_eq!(bad_id["error"]["code"], -32600);
        assert!(bad_id["id"].is_null());
    }

    #[test]
    fn notification_before_initialize_does_not_unlock_tools() {
        let mut server = Server::default();
        // initialized notification arrives with no prior initialize request.
        server.handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        let response = server
            .handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }))
            .unwrap();
        assert_eq!(response["error"]["code"], -32002);
    }

    #[test]
    fn non_object_arguments_are_rejected() {
        let response = call_tool(
            json!(1),
            &json!({ "name": "list_ports", "arguments": "nope" }),
        );
        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn tools_list_exposes_the_three_tools() {
        let mut server = initialized_server();
        let response = server
            .handle(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
            .unwrap();
        let names: Vec<&str> = response["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["list_ports", "inspect_port", "free_port"]);
    }

    #[test]
    fn negotiate_prefers_supported_else_default() {
        assert_eq!(negotiate_protocol_version(Some("2025-03-26")), "2025-03-26");
        assert_eq!(
            negotiate_protocol_version(Some("bogus")),
            SUPPORTED_PROTOCOL_VERSIONS[0]
        );
        assert_eq!(
            negotiate_protocol_version(None),
            SUPPORTED_PROTOCOL_VERSIONS[0]
        );
    }

    #[test]
    fn optional_bool_rejects_wrong_type() {
        assert_eq!(
            optional_bool(&json!({ "includeUdp": true }), "includeUdp"),
            Ok(Some(true))
        );
        assert_eq!(optional_bool(&json!({}), "includeUdp"), Ok(None));
        assert!(optional_bool(&json!({ "includeUdp": "true" }), "includeUdp").is_err());
    }

    #[test]
    fn summary_hides_paths_and_command() {
        let value = summarize(&sample(3000, false));
        assert!(value.get("command").is_none());
        assert!(value.get("executablePath").is_none());
        assert!(value.get("workingDirectory").is_none());
        assert_eq!(value["port"], 3000);
        assert_eq!(value["protocol"], "tcp");
    }

    #[test]
    fn detail_exposes_paths_and_command() {
        let value = detail(&sample(3000, false));
        assert_eq!(value["command"], "node server.js --secret");
        assert_eq!(value["executablePath"], "C:/proj/node.exe");
        assert_eq!(value["workingDirectory"], "C:/proj");
    }

    #[test]
    fn parse_port_rejects_out_of_range_and_zero() {
        assert!(parse_port(&json!({ "port": 3000 })).is_ok());
        assert!(parse_port(&json!({ "port": 0 })).is_err());
        assert!(parse_port(&json!({ "port": 70000 })).is_err());
        assert!(parse_port(&json!({})).is_err());
    }

    #[test]
    fn tool_error_is_reported_as_tool_result_not_protocol_error() {
        let result = tool_result(&Value::String("protected port".into()), true);
        assert_eq!(result["isError"], true);
        assert_eq!(result["content"][0]["text"], "protected port");
        assert_eq!(result["content"][0]["type"], "text");
    }
}
