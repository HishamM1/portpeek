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

/// A validated, reproducible relaunch plan captured server-side.
///
/// The program is spawned directly with its real argument vector — never
/// re-joined into a string and re-split through a shell — so argument
/// boundaries and quoting (e.g. `Program Files` paths, spaced arguments) are
/// preserved across the restart.
#[cfg(target_os = "windows")]
#[derive(Debug, PartialEq)]
struct Relaunch {
    program: std::path::PathBuf,
    args: Vec<std::ffi::OsString>,
    working_directory: std::path::PathBuf,
    environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

/// Reconstruct a safely-reproducible invocation from data captured server-side
/// via `sysinfo` (executable image path, argv, cwd). Returns an error — so the
/// caller can refuse to restart *before* terminating anything — when the
/// invocation cannot be reproduced.
///
/// We spawn the resolved executable image (`exe`) directly with the real
/// argument vector (`argv[1..]`, `OsString`s passed through verbatim so no
/// quoting/boundary information is lost). We deliberately do **not** fall back
/// to `argv[0]`: it may be a bare name or a path relative to the process's
/// *original* cwd/PATH, which — after we've already terminated the original —
/// could fail to launch or launch a *different* program (kill-without-relaunch).
/// If a resolved absolute executable is unavailable we refuse here, before
/// anything is terminated.
#[cfg(target_os = "windows")]
fn plan_relaunch(
    exe: Option<&std::path::Path>,
    argv: &[std::ffi::OsString],
    cwd: Option<&std::path::Path>,
    environ: &[std::ffi::OsString],
) -> Result<Relaunch, String> {
    if argv.is_empty() {
        return Err("Failed to restart process: process command line is unavailable".into());
    }

    let program = match exe {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        _ => {
            return Err(
                "Failed to restart process: cannot resolve a reproducible executable path".into(),
            )
        }
    };

    let working_directory = match cwd {
        Some(path) if !path.as_os_str().is_empty() => path.to_path_buf(),
        _ => {
            return Err(
                "Failed to restart process: process working directory is unavailable".into(),
            )
        }
    };

    Ok(Relaunch {
        program,
        args: argv[1..].to_vec(),
        working_directory,
        environment: parse_environ(environ),
    })
}

#[cfg(target_os = "windows")]
fn spawn_relaunch(plan: &Relaunch) -> std::io::Result<std::process::Child> {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    let mut cmd = std::process::Command::new(&plan.program);
    cmd.args(&plan.args);
    cmd.current_dir(&plan.working_directory);
    if !plan.environment.is_empty() {
        cmd.env_clear();
        cmd.envs(plan.environment.iter().cloned());
    }
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.spawn()
}

#[cfg(target_os = "windows")]
fn parse_environ(environ: &[std::ffi::OsString]) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    environ
        .iter()
        .filter_map(|entry| {
            let entry = entry.to_string_lossy();
            let (key, value) = entry.split_once('=')?;
            (!key.is_empty()).then(|| (key.into(), value.into()))
        })
        .collect()
}

#[tauri::command]
pub fn restart_process(pid: u32) -> Result<(), String> {
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
                .with_user(UpdateKind::Always)
                .with_cmd(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_environ(UpdateKind::Always),
        );
        let process = system
            .process(pid_sys)
            .ok_or_else(|| "Failed to restart process: process no longer exists".to_string())?;

        let process_name = process.name().to_string_lossy().into_owned();
        let sid = process.user_id().map(|uid| uid.to_string());

        if crate::platform::windows::processes::is_system_process(
            sid.as_deref(),
            process.exe(),
            &process_name,
            pid,
        ) {
            return Err("Failed to restart process: refusing to restart a system process".into());
        }

        // Keep argv as OsStrings so argument boundaries/quoting survive verbatim
        // (no lossy String round-trip before we relaunch).
        let argv: Vec<std::ffi::OsString> = process.cmd().to_vec();
        let environ: Vec<std::ffi::OsString> = process.environ().to_vec();

        // Capture a reproducible invocation BEFORE terminating anything. If it
        // can't be reproduced we bail out here, so we never kill-without-relaunch.
        let plan = plan_relaunch(process.exe(), &argv, process.cwd(), &environ)?;

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

        let mut child = spawn_relaunch(&plan)
            .map_err(|error| format!("Failed to relaunch process: {error}"))?;

        // ponytail: confirm the relaunch started and survived a moment, not that
        // it is actually listening on the port again. Upgrade path: re-scan the
        // port after a grace period and report listening status.
        std::thread::sleep(std::time::Duration::from_millis(200));
        match child.try_wait() {
            Ok(Some(status)) => Err(format!(
                "Failed to relaunch process: process exited immediately with status {status}"
            )),
            Ok(None) => Ok(()),
            Err(err) => Err(format!(
                "Failed to relaunch process: could not verify process status: {err}"
            )),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
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

    #[cfg(target_os = "windows")]
    fn osv(parts: &[&str]) -> Vec<std::ffi::OsString> {
        parts.iter().map(std::ffi::OsString::from).collect()
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn plan_relaunch_uses_resolved_exe_and_real_args() {
        use std::path::{Path, PathBuf};
        // argv[0] is a bare name (as it often is: `node server.js`); the resolved
        // absolute image path must be the program, so we don't depend on PortPeek's PATH.
        let argv = osv(&["node", "server.js"]);
        let exe = Path::new("C:\\Program Files\\nodejs\\node.exe");
        let cwd = Path::new("C:\\Projects\\app");
        let plan = plan_relaunch(Some(exe), &argv, Some(cwd), &[]).expect("should reconstruct");
        assert_eq!(
            plan.program,
            PathBuf::from("C:\\Program Files\\nodejs\\node.exe")
        );
        // argv[1..] are the real arguments, preserved verbatim (boundaries intact).
        assert_eq!(plan.args, osv(&["server.js"]));
        assert_eq!(plan.working_directory, PathBuf::from("C:\\Projects\\app"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn plan_relaunch_preserves_spaced_arguments() {
        use std::path::Path;
        // A spaced path lives in a single argv element and must stay one arg —
        // no re-joining/re-splitting through a shell.
        let argv = osv(&[
            "node",
            "C:\\My Projects\\app\\server.js",
            "--config",
            "C:\\My Projects\\app\\vite config.js",
        ]);
        let plan = plan_relaunch(
            Some(Path::new("C:\\Program Files\\nodejs\\node.exe")),
            &argv,
            Some(Path::new("C:\\My Projects\\app")),
            &[],
        )
        .expect("should reconstruct");
        assert_eq!(plan.args, argv[1..].to_vec());
        assert_eq!(
            plan.args[0],
            std::ffi::OsString::from("C:\\My Projects\\app\\server.js")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn plan_relaunch_refuses_without_absolute_exe() {
        use std::path::Path;
        let argv = osv(&["svc.exe", "--flag"]);
        // Missing image path => not safely reproducible; refuse (caller bails BEFORE terminating).
        let err = plan_relaunch(None, &argv, Some(Path::new("C:\\work")), &[]).unwrap_err();
        assert!(err.contains("cannot resolve a reproducible executable path"));
        // A relative argv[0]-style path is not accepted as the executable either.
        let err = plan_relaunch(
            Some(Path::new("svc.exe")),
            &argv,
            Some(Path::new("C:\\work")),
            &[],
        )
        .unwrap_err();
        assert!(err.contains("cannot resolve a reproducible executable path"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn plan_relaunch_refuses_missing_cwd() {
        // No working directory => not safely reproducible; must error BEFORE any
        // terminate happens (the caller propagates this before killing).
        let argv = osv(&["node", "server.js"]);
        let err = plan_relaunch(
            Some(std::path::Path::new("C:\\bin\\node.exe")),
            &argv,
            None,
            &[],
        )
        .unwrap_err();
        assert!(err.contains("working directory is unavailable"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn plan_relaunch_refuses_empty_command_line() {
        let err =
            plan_relaunch(None, &[], Some(std::path::Path::new("C:\\work")), &[]).unwrap_err();
        assert!(err.contains("command line is unavailable"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_environ_splits_pairs_and_skips_special_entries() {
        use std::ffi::OsString;
        let environ = vec![
            OsString::from("PATH=C:\\a;C:\\b"),
            OsString::from("NODE_ENV=development"),
            OsString::from("EQ=a=b=c"),
            OsString::from("=C:=C:\\somewhere"),
            OsString::from("NOEQUALS"),
        ];
        assert_eq!(
            parse_environ(&environ),
            vec![
                (OsString::from("PATH"), OsString::from("C:\\a;C:\\b")),
                (OsString::from("NODE_ENV"), OsString::from("development")),
                (OsString::from("EQ"), OsString::from("a=b=c")),
            ]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "spawns real node processes; run manually with --ignored"]
    fn restart_relaunch_rebinds_the_port_end_to_end() {
        use std::net::TcpListener;
        use std::time::{Duration, Instant};
        use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let script = format!("require('net').createServer().listen({port}, '127.0.0.1')");

        let mut original = std::process::Command::new("node")
            .args(["-e", &script])
            .spawn()
            .expect("node should be on PATH");
        let pid = original.id();

        let listening = || {
            crate::platform::windows::ports::list_tcp_listeners()
                .map(|items| items.iter().any(|item| item.port == port))
                .unwrap_or(false)
        };

        let deadline = Instant::now() + Duration::from_secs(8);
        while Instant::now() < deadline && !listening() {
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(listening(), "original node never bound port {port}");

        let mut system = System::new();
        let pid_sys = Pid::from_u32(pid);
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid_sys]),
            true,
            ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::Always)
                .with_cmd(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_environ(UpdateKind::Always),
        );
        let process = system.process(pid_sys).expect("process visible to sysinfo");
        let argv: Vec<std::ffi::OsString> = process.cmd().to_vec();
        let environ: Vec<std::ffi::OsString> = process.environ().to_vec();
        let plan = plan_relaunch(process.exe(), &argv, process.cwd(), &environ)
            .expect("invocation should be reproducible");

        crate::platform::windows::processes::terminate(pid).expect("terminate original");
        let _ = original.wait();

        let mut relaunched = spawn_relaunch(&plan).expect("relaunch should spawn");
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && !listening() {
            std::thread::sleep(Duration::from_millis(100));
        }
        let rebound = listening();

        let _ = relaunched.kill();
        let _ = relaunched.wait();

        assert!(rebound, "relaunched node did not rebind port {port}");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn restart_process_validates_protected_processes() {
        // Attempting to restart PID 4 (System) must be rejected and must never
        // terminate it. Accept either the system-process refusal or a clean
        // "no longer exists" (if this environment cannot enumerate PID 4) —
        // both prove PID 4 was not restarted.
        let result = restart_process(4);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("refusing to restart a system process")
                || err.contains("refusing to terminate a protected process")
                || err.contains("process no longer exists"),
            "unexpected error for PID 4: {err}"
        );

        // A non-existent PID must fail cleanly before any relaunch attempt.
        let result = restart_process(999999);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("process no longer exists"));
    }
}
