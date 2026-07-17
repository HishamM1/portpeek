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

#[tauri::command]
pub fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_in_editor(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, Some("code"))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn detect_vscode() -> bool {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::editors::has_vscode()
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[tauri::command]
pub fn restart_process(pid: u32, command: String, working_directory: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
        let mut system = System::new();
        let pid_sys = Pid::from_u32(pid);
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid_sys]),
            true,
            ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::Always)
                .with_user(UpdateKind::Always),
        );
        let process = system.process(pid_sys)
            .ok_or_else(|| "Failed to stop process: process no longer exists".to_string())?;

        let process_name = process.name().to_string_lossy().into_owned();
        let sid = process.user_id().map(|uid| uid.to_string());

        if crate::platform::windows::processes::is_system_process(sid.as_deref(), process.exe(), &process_name, pid) {
            return Err("Failed to stop process: refusing to restart a system process".into());
        }

        crate::platform::windows::processes::terminate(pid)
            .map_err(|error| format!("Failed to stop process: {error}"))?;

        let mut exited = false;
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid_sys]),
                false,
                ProcessRefreshKind::nothing(),
            );
            if system.process(pid_sys).is_none() {
                exited = true;
                break;
            }
        }
        if !exited {
            return Err("Failed to stop process: process did not exit within timeout".into());
        }

        let mut cmd = std::process::Command::new("cmd.exe");
        cmd.arg("/c").arg(&command);
        cmd.current_dir(&working_directory);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.spawn()
            .map_err(|error| format!("Failed to relaunch process: {error}"))?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (pid, command, working_directory);
        Err("process restart is currently supported on Windows only".into())
    }
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

    #[test]
    fn restart_process_validates_protected_processes() {
        // Attempting to restart PID 4 (System) should be rejected
        let result = restart_process(4, "cmd.exe".to_string(), "C:\\".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("refusing to restart a system process") || err.contains("refusing to terminate a protected process"));

        // Attempting to restart a non-existent PID should fail cleanly
        let result = restart_process(999999, "cmd.exe".to_string(), "C:\\".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("process no longer exists"));
    }
}
