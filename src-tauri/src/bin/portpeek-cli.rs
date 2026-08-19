use clap::{Parser, Subcommand};
use portpeek_lib::domain::ports::types::{PortItem, PortProtocol};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "portpeek", version, about = "See what's listening on your local ports.")]
struct Cli {
    /// Show details for the process listening on this port
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<Command>,

    /// Include system/OS-owned ports in the listing
    #[arg(short = 'a', long, global = true)]
    all: bool,

    /// Include UDP listeners in the listing
    #[arg(long, global = true)]
    udp: bool,

    /// Print machine-readable JSON instead of a table
    #[arg(long, global = true)]
    json: bool,

    #[arg(long, hide = true)]
    install_path: Option<PathBuf>,

    #[arg(long, hide = true)]
    remove_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Stop the process listening on a port
    Free {
        /// The port to free
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = if let Some(path) = cli.install_path {
        update_user_path(&path, true)
    } else if let Some(path) = cli.remove_path {
        update_user_path(&path, false)
    } else {
        match cli.command {
            Some(Command::Free { port }) => free(port, cli.json),
            None => match cli.port {
                Some(port) => show(port, cli.json),
                None => list(cli.all, cli.udp, cli.json),
            },
        }
    };

    if let Err(message) = result {
        eprintln!("error: {message}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "windows")]
fn update_user_path(path: &std::path::Path, install: bool) -> Result<(), String> {
    let script = if install {
        r#"$entry=$env:PORTPEEK_CLI_PATH; $path=[Environment]::GetEnvironmentVariable('Path','User'); if ($null -eq $path) {$path=''}; $parts=@($path.Split([char]';',[StringSplitOptions]::None)); if ($parts -notcontains $entry) { $new=if ([string]::IsNullOrEmpty($path)) {$entry} elseif ($path.EndsWith(';')) {"$path$entry;"} else {"$path;$entry"}; [Environment]::SetEnvironmentVariable('Path',$new,'User') }"#
    } else {
        r#"$entry=$env:PORTPEEK_CLI_PATH; $path=[Environment]::GetEnvironmentVariable('Path','User'); if ($null -eq $path) {$path=''}; $parts=@($path.Split([char]';',[StringSplitOptions]::None) | Where-Object { $_ -ne $entry }); [Environment]::SetEnvironmentVariable('Path',($parts -join ';'),'User')"#
    };
    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("PORTPEEK_CLI_PATH", path)
        .status()
        .map_err(|error| format!("failed to update PATH: {error}"))?;
    status.success().then_some(()).ok_or_else(|| {
        format!(
            "failed to update PATH (exit code {})",
            status.code().unwrap_or(-1)
        )
    })
}

#[cfg(not(target_os = "windows"))]
fn update_user_path(_path: &std::path::Path, _install: bool) -> Result<(), String> {
    Err("PATH integration is supported on Windows only".into())
}

fn list(all: bool, udp: bool, json: bool) -> Result<(), String> {
    let mut items = scan(udp)?;
    if !all {
        items.retain(|item| !item.is_system_port);
    }

    let rows = collapse(&items);
    if json {
        print_json(&rows)
    } else {
        print_table(&rows);
        Ok(())
    }
}

fn show(port: u16, json: bool) -> Result<(), String> {
    let items = matching(port)?;
    if items.is_empty() {
        return Err(format!("nothing is listening on port {port}"));
    }

    let rows = collapse(&items);
    if json {
        return print_json(&rows);
    }

    for row in &rows {
        print_details(row);
    }
    Ok(())
}

fn free(port: u16, json: bool) -> Result<(), String> {
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

    if json {
        let payload = serde_json::json!({
            "port": port,
            "freedPids": freed,
            "errors": errors,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?
        );
    } else if freed.is_empty() {
        println!("Failed to free port {port}.");
    } else {
        let pid_list = freed.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
        println!("Freed port {port} (stopped pid {pid_list}).");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
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

/// One listener, with every address it is bound to. The same process on the same
/// port shows up once per bind address in the OS tables (127.0.0.1 and ::1, say),
/// and those rows are identical apart from the address.
struct Row<'a> {
    item: &'a PortItem,
    addresses: Vec<&'a str>,
}

impl Row<'_> {
    fn address_cell(&self) -> String {
        if self.addresses.iter().all(|address| matches!(*address, "0.0.0.0" | "::")) {
            "*".to_string()
        } else {
            self.addresses.join(", ")
        }
    }

    fn id(&self) -> String {
        let protocol = protocol_label(self.item.protocol);
        match self.item.pid {
            Some(pid) => format!("{protocol}|{}|{pid}", self.item.port),
            None => format!("{protocol}|{}", self.item.port),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value, String> {
        let mut value = serde_json::to_value(self.item).map_err(|error| error.to_string())?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| "expected a port object".to_string())?;
        object.remove("address");
        object.insert("id".into(), self.id().into());
        object.insert("addresses".into(), self.addresses.clone().into());
        Ok(value)
    }
}

fn collapse(items: &[PortItem]) -> Vec<Row<'_>> {
    let mut rows: Vec<Row> = Vec::new();
    let mut index: HashMap<_, usize> = HashMap::new();
    for item in items {
        let key = (item.port, protocol_label(item.protocol), item.pid);
        match index.get(&key) {
            Some(&position) => rows[position].addresses.push(&item.address),
            None => {
                index.insert(key, rows.len());
                rows.push(Row {
                    item,
                    addresses: vec![&item.address],
                });
            }
        }
    }
    rows
}

fn print_json(rows: &[Row]) -> Result<(), String> {
    let payload = rows.iter().map(Row::to_json).collect::<Result<Vec<_>, _>>()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn print_table(rows: &[Row]) {
    if rows.is_empty() {
        println!("No listening ports found.");
        return;
    }

    let cells: Vec<[String; 6]> = rows
        .iter()
        .map(|row| {
            let item = row.item;
            [
                item.port.to_string(),
                protocol_label(item.protocol).to_string(),
                row.address_cell(),
                item.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "—".into()),
                item.display_name
                    .clone()
                    .or_else(|| item.process_name.clone())
                    .unwrap_or_else(|| "—".into()),
                format_memory(item.memory_mb),
            ]
        })
        .collect();

    let headers = ["PORT", "PROTO", "ADDRESS", "PID", "PROCESS", "MEM"];
    let mut widths = headers.map(display_width);
    for row in &cells {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(display_width(cell));
        }
    }

    print_row(&headers.map(str::to_string), &widths);
    for row in &cells {
        print_row(row, &widths);
    }
}

fn display_width(cell: &str) -> usize {
    // ponytail: char count, not bytes — the "—" placeholder is 3 bytes wide and
    // would over-pad its column. Good enough short of full grapheme handling.
    cell.chars().count()
}

fn print_row(cells: &[String; 6], widths: &[usize; 6]) {
    let line = cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| format!("{cell:<width$}"))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}", line.trim_end());
}

fn print_details(row: &Row) {
    let item = row.item;
    println!("port:        {} ({})", item.port, protocol_label(item.protocol));
    println!("address:     {}", row.address_cell());
    println!(
        "process:     {}",
        item.display_name
            .as_deref()
            .or(item.process_name.as_deref())
            .unwrap_or("—")
    );
    println!("pid:         {}", item.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "—".into()));
    println!("memory:      {}", format_memory(item.memory_mb));
    println!("uptime:      {}", format_uptime(item.uptime_seconds));
    println!("executable:  {}", item.executable_path.as_deref().unwrap_or("—"));
    println!("project:     {}", item.working_directory.as_deref().unwrap_or("—"));
    println!("command:     {}", item.command.as_deref().unwrap_or("—"));
    println!();
}

fn protocol_label(protocol: PortProtocol) -> &'static str {
    match protocol {
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    }
}

fn format_memory(memory_mb: Option<f64>) -> String {
    match memory_mb {
        None => "—".to_string(),
        Some(mb) if mb < 10.0 => format!("{mb:.1} MB"),
        Some(mb) => format!("{} MB", mb.round() as i64),
    }
}

fn format_uptime(seconds: Option<u64>) -> String {
    let Some(seconds) = seconds else {
        return "—".to_string();
    };
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86_400 {
        format!("{}h {}m", seconds / 3_600, (seconds % 3_600) / 60)
    } else {
        format!("{}d {}h", seconds / 86_400, (seconds % 86_400) / 3_600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_list_invocation() {
        let cli = Cli::parse_from(["portpeek"]);
        assert!(cli.port.is_none());
        assert!(cli.command.is_none());
        assert!(!cli.all);
        assert!(!cli.udp);
        assert!(!cli.json);
        assert!(cli.install_path.is_none());
        assert!(cli.remove_path.is_none());
    }

    #[test]
    fn parses_a_port_lookup() {
        let cli = Cli::parse_from(["portpeek", "3000"]);
        assert_eq!(cli.port, Some(3000));
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_free_subcommand() {
        let cli = Cli::parse_from(["portpeek", "free", "3000"]);
        assert!(matches!(cli.command, Some(Command::Free { port: 3000 })));
    }

    #[test]
    fn parses_flags() {
        let cli = Cli::parse_from(["portpeek", "--all", "--udp", "--json"]);
        assert!(cli.all);
        assert!(cli.udp);
        assert!(cli.json);
    }

    #[test]
    fn parses_short_all_flag() {
        let cli = Cli::parse_from(["portpeek", "-a"]);
        assert!(cli.all);
    }

    fn listener(address: &str, port: u16, pid: u32) -> PortItem {
        PortItem {
            id: format!("tcp|{address}|{port}|{pid}"),
            port,
            address: address.into(),
            protocol: PortProtocol::Tcp,
            pid: Some(pid),
            process_name: Some("postgres.exe".into()),
            display_name: Some("postgres".into()),
            memory_mb: Some(5.6),
            uptime_seconds: None,
            command: None,
            executable_path: None,
            working_directory: None,
            url: None,
            favicon_url: None,
            cached_favicon_path: None,
            framework: None,
            is_system_port: false,
        }
    }

    #[test]
    fn collapses_ipv4_ipv6_into_one_row() {
        let items = [
            listener("127.0.0.1", 5432, 3008),
            listener("::1", 5432, 3008),
        ];
        let rows = collapse(&items);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].address_cell(), "127.0.0.1, ::1");
        assert_eq!(rows[0].id(), "tcp|5432|3008");
    }

    #[test]
    fn keeps_distinct_pids_apart() {
        let items = [
            listener("127.0.0.1", 5432, 3008),
            listener("0.0.0.0", 5432, 4011),
        ];
        assert_eq!(collapse(&items).len(), 2);
    }

    #[test]
    fn shows_wildcard_binds_as_a_star() {
        let items = [listener("0.0.0.0", 22, 6380), listener("::", 22, 6380)];
        assert_eq!(collapse(&items)[0].address_cell(), "*");
    }

    #[test]
    fn json_replaces_address_with_an_addresses_array() {
        let items = [
            listener("127.0.0.1", 5432, 3008),
            listener("::1", 5432, 3008),
        ];
        let value = collapse(&items)[0].to_json().unwrap();
        assert!(value.get("address").is_none());
        assert_eq!(value["addresses"], serde_json::json!(["127.0.0.1", "::1"]));
        assert_eq!(value["id"], "tcp|5432|3008");
        assert_eq!(value["port"], 5432);
    }

    #[test]
    fn column_width_counts_characters_not_bytes() {
        assert_eq!(display_width("—"), 1);
        assert_eq!(display_width("127.0.0.1, ::1"), 14);
    }

    #[test]
    fn parses_hidden_path_helpers() {
        let install = Cli::parse_from(["portpeek", "--install-path", r"C:\PortPeek\bin"]);
        assert_eq!(
            install.install_path,
            Some(PathBuf::from(r"C:\PortPeek\bin"))
        );

        let remove = Cli::parse_from(["portpeek", "--remove-path", r"C:\PortPeek\bin"]);
        assert_eq!(remove.remove_path, Some(PathBuf::from(r"C:\PortPeek\bin")));
    }

    #[test]
    fn formats_memory_like_the_gui() {
        assert_eq!(format_memory(None), "—");
        assert_eq!(format_memory(Some(4.2)), "4.2 MB");
        assert_eq!(format_memory(Some(128.0)), "128 MB");
    }
}
