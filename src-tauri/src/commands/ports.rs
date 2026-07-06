use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;

use crate::domain::{ports::types::PortItem, settings::types::OpenProtocol};

#[tauri::command]
pub fn list_ports(app: AppHandle) -> Result<Vec<PortItem>, String> {
    #[cfg(target_os = "windows")]
    {
        let mut items = crate::platform::windows::ports::list_tcp_listeners()
            .map_err(|error| format!("failed to list TCP listeners: {error}"))?;
        let mut udp = crate::platform::windows::ports::list_udp_listeners()
            .map_err(|error| format!("failed to list UDP listeners: {error}"))?;
        items.append(&mut udp);
        items.sort_by(|left, right| {
            left.port
                .cmp(&right.port)
                .then_with(|| left.address.cmp(&right.address))
                .then_with(|| left.pid.cmp(&right.pid))
        });
        crate::domain::detection::enrich(&app, &mut items);
        Ok(items)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("port discovery is currently supported on Windows only".into())
    }
}

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::processes::terminate(pid)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Err("process termination is currently supported on Windows only".into())
    }
}

#[tauri::command]
pub fn kill_process_elevated(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::processes::terminate_elevated(pid)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Err("process termination is currently supported on Windows only".into())
    }
}

#[tauri::command]
pub fn open_localhost_url(app: AppHandle, port: u16, protocol: OpenProtocol) -> Result<(), String> {
    app.opener()
        .open_url(localhost_url(port, protocol), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn copy_localhost_url(app: AppHandle, port: u16, protocol: OpenProtocol) -> Result<(), String> {
    app.clipboard()
        .write_text(localhost_url(port, protocol))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn copy_port(app: AppHandle, port: u16) -> Result<(), String> {
    app.clipboard()
        .write_text(port.to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|error| error.to_string())
}

fn localhost_url(port: u16, protocol: OpenProtocol) -> String {
    format!("{}://localhost:{port}", protocol.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_safe_localhost_urls() {
        assert_eq!(
            localhost_url(5173, OpenProtocol::Http),
            "http://localhost:5173"
        );
    }
}
