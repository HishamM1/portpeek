use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

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
            .with_user(UpdateKind::Always)
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
        let sid = process.user_id().map(|uid| uid.to_string());
        item.is_system_port = is_system_process(sid.as_deref(), process.exe(), &process_name, pid);
    }
}

fn validate_killable(pid: u32) -> Result<(), String> {
    if pid <= 4 || pid == std::process::id() {
        return Err("refusing to terminate a protected process".into());
    }

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
        ProcessRefreshKind::nothing(),
    );
    let name = system
        .process(Pid::from_u32(pid))
        .ok_or_else(|| "process no longer exists".to_string())?
        .name()
        .to_string_lossy()
        .into_owned();
    if protected_name(&name) {
        return Err(format!("refusing to terminate protected process {name}"));
    }
    Ok(())
}

pub fn terminate(pid: u32) -> Result<(), String> {
    validate_killable(pid)?;

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

pub fn terminate_elevated(pid: u32) -> Result<(), String> {
    validate_killable(pid)?;

    let system32 = format!(
        "{}\\System32",
        std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into())
    );
    let powershell = format!("{system32}\\WindowsPowerShell\\v1.0\\powershell.exe");
    let taskkill = format!("{system32}\\taskkill.exe");
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let status = Command::new(&powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &format!(
                "Start-Process -FilePath '{taskkill}' -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList '/PID','{pid}','/F'"
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err("elevation was cancelled".into());
    }

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::nothing(),
    );
    if system.process(Pid::from_u32(pid)).is_some() {
        return Err("access is denied".into());
    }
    Ok(())
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

// A listener is a "system" port when its owning process is a Windows OS process:
// running under a built-in system account, the kernel (pid <= 4), or an executable
// inside the Windows directory. Owner account is the authoritative signal.
pub(crate) fn is_system_process(sid: Option<&str>, exe: Option<&Path>, name: &str, pid: u32) -> bool {
    pid <= 4
        || matches!(sid, Some("S-1-5-18" | "S-1-5-19" | "S-1-5-20"))
        || exe.is_some_and(is_under_system_root)
        || protected_name(name)
}

fn is_under_system_root(exe: &Path) -> bool {
    let root = std::env::var("SystemRoot")
        .unwrap_or_else(|_| "C:\\Windows".to_string())
        .to_ascii_lowercase();
    exe.to_string_lossy().to_ascii_lowercase().starts_with(&root)
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
    fn classifies_system_by_account_and_location() {
        // Built-in system accounts (SYSTEM / LOCAL SERVICE / NETWORK SERVICE)
        assert!(is_system_process(Some("S-1-5-18"), None, "svchost.exe", 900));
        assert!(is_system_process(Some("S-1-5-19"), None, "spoolsv.exe", 900));
        // Kernel
        assert!(is_system_process(None, None, "System", 4));
        // Executable inside the Windows directory, even without an SID
        let sys_exe = format!(
            "{}\\System32\\spoolsv.exe",
            std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into())
        );
        assert!(is_system_process(None, Some(Path::new(&sys_exe)), "spoolsv.exe", 900));
        // A normal dev server owned by the user, running from a project folder
        assert!(!is_system_process(
            Some("S-1-5-21-1-2-3-1001"),
            Some(Path::new("C:\\Projects\\shop\\node.exe")),
            "node.exe",
            5173,
        ));
    }
}
