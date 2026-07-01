use std::path::Path;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE},
};

use crate::domain::ports::types::PortItem;

pub fn enrich(items: &mut [PortItem]) {
    let mut pids: Vec<Pid> = items
        .iter()
        .filter_map(|item| item.pid)
        .map(Pid::from_u32)
        .collect();
    pids.sort_unstable();
    pids.dedup();

    if pids.is_empty() {
        return;
    }

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        true,
        ProcessRefreshKind::nothing()
            .with_memory()
            .with_cwd(UpdateKind::Always)
            .with_cmd(UpdateKind::Always)
            .with_exe(UpdateKind::Always)
            .without_tasks(),
    );

    for item in items {
        let Some(pid) = item.pid else { continue };
        let Some(process) = system.process(Pid::from_u32(pid)) else {
            continue;
        };

        let process_name = process.name().to_string_lossy().into_owned();
        let working_directory = process.cwd().map(path_string);
        item.display_name = display_name(&process_name, process.cwd());
        item.process_name = Some(process_name.clone());
        item.memory_mb = Some(process.memory() as f64 / 1_048_576.0);
        item.uptime_seconds = Some(process.run_time());
        item.command = (!process.cmd().is_empty()).then(|| {
            process
                .cmd()
                .iter()
                .map(|part| part.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        });
        item.executable_path = process.exe().map(path_string);
        item.working_directory = working_directory;
        item.is_system_port = is_system_process(&process_name, item.port, pid);
    }
}

pub fn terminate(pid: u32) -> Result<(), String> {
    if pid <= 4 || pid == std::process::id() {
        return Err("refusing to terminate a protected process".into());
    }

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
        ProcessRefreshKind::nothing(),
    );
    let process = system
        .process(Pid::from_u32(pid))
        .ok_or_else(|| "process no longer exists".to_string())?;
    let name = process.name().to_string_lossy();
    if protected_name(&name) {
        return Err(format!("refusing to terminate protected process {name}"));
    }

    // SAFETY: the PID is validated above; the returned handle is checked and
    // always closed after the single TerminateProcess call.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }

        let terminated = TerminateProcess(handle, 1);
        let error = (terminated == 0).then(std::io::Error::last_os_error);
        CloseHandle(handle);

        match error {
            Some(error) => Err(error.to_string()),
            None => Ok(()),
        }
    }
}

fn display_name(process_name: &str, cwd: Option<&Path>) -> Option<String> {
    let generic = [
        "node.exe",
        "python.exe",
        "python3.exe",
        "ruby.exe",
        "php.exe",
    ];
    if generic.contains(&process_name.to_ascii_lowercase().as_str()) {
        return cwd
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned());
    }

    Some(process_name.trim_end_matches(".exe").to_string())
}

fn is_system_process(name: &str, port: u16, pid: u32) -> bool {
    port < 1024 || pid <= 4 || protected_name(name)
}

fn protected_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "system"
            | "registry"
            | "smss.exe"
            | "csrss.exe"
            | "wininit.exe"
            | "services.exe"
            | "lsass.exe"
            | "winlogon.exe"
            | "svchost.exe"
    )
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_windows_services_and_privileged_ports() {
        assert!(is_system_process("svchost.exe", 5173, 900));
        assert!(is_system_process("node.exe", 80, 900));
        assert!(!is_system_process("node.exe", 5173, 900));
    }
}
