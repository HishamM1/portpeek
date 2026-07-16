use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Whether the `code` launcher (VS Code) is resolvable on PATH.
pub fn has_vscode() -> bool {
    Command::new("cmd")
        .args(["/c", "where", "code"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
