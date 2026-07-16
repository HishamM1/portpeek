use clap::{Parser, Subcommand};
use portpeek_lib::domain::ports::types::{PortItem, PortProtocol};

#[derive(Parser)]
#[command(name = "portpeek", version, about = "See what's listening on your local ports.")]
struct Cli {
    /// Show details for the process listening on this port
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<Command>,

    /// Include system/OS-owned ports in the listing
    #[arg(long, global = true)]
    all: bool,

    /// Include UDP listeners in the listing
    #[arg(long, global = true)]
    udp: bool,

    /// Print machine-readable JSON instead of a table
    #[arg(long, global = true)]
    json: bool,
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

    let result = match cli.command {
        Some(Command::Free { port }) => free(port, cli.json),
        None => match cli.port {
            Some(port) => show(port, cli.json),
            None => list(cli.all, cli.udp, cli.json),
        },
    };

    if let Err(message) = result {
        eprintln!("error: {message}");
        std::process::exit(1);
    }
}

fn list(all: bool, udp: bool, json: bool) -> Result<(), String> {
    let mut items = scan(udp)?;
    if !all {
        items.retain(|item| !item.is_system_port);
    }

    if json {
        print_json(&items)
    } else {
        print_table(&items);
        Ok(())
    }
}

fn show(port: u16, json: bool) -> Result<(), String> {
    let items = matching(port)?;
    if items.is_empty() {
        return Err(format!("nothing is listening on port {port}"));
    }

    if json {
        return print_json(&items);
    }

    for item in &items {
        print_details(item);
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

fn print_json(items: &[PortItem]) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(items).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn print_table(items: &[PortItem]) {
    if items.is_empty() {
        println!("No listening ports found.");
        return;
    }

    let rows: Vec<[String; 5]> = items
        .iter()
        .map(|item| {
            [
                item.port.to_string(),
                protocol_label(item.protocol).to_string(),
                item.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "—".into()),
                item.display_name
                    .clone()
                    .or_else(|| item.process_name.clone())
                    .unwrap_or_else(|| "—".into()),
                format_memory(item.memory_mb),
            ]
        })
        .collect();

    let headers = ["PORT", "PROTO", "PID", "PROCESS", "MEM"];
    let mut widths = headers.map(str::len);
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.len());
        }
    }

    print_row(&headers.map(str::to_string), &widths);
    for row in &rows {
        print_row(row, &widths);
    }
}

fn print_row(cells: &[String; 5], widths: &[usize; 5]) {
    let line = cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| format!("{cell:<width$}"))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}", line.trim_end());
}

fn print_details(item: &PortItem) {
    println!("port:        {} ({})", item.port, protocol_label(item.protocol));
    println!("address:     {}", item.address);
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
    fn formats_memory_like_the_gui() {
        assert_eq!(format_memory(None), "—");
        assert_eq!(format_memory(Some(4.2)), "4.2 MB");
        assert_eq!(format_memory(Some(128.0)), "128 MB");
    }
}
