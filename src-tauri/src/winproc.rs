//! Running Windows' own tools (gpupdate, ipconfig, powershell, explorer) from
//! a GUI app: absolute paths under %WINDIR% (no PATH hijacking), no console
//! window flashing up, and off the async runtime because they block.

/// `tauri::async_runtime::spawn_blocking` with the join error as a message.
pub async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("Hintergrundaufgabe abgebrochen: {e}"))
}

/// A hidden-window command for `relative_path` below %WINDIR%,
/// e.g. `r"System32\gpupdate.exe"`.
#[cfg(windows)]
pub fn windows_tool(relative_path: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let mut command = std::process::Command::new(std::path::Path::new(&windir).join(relative_path));
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// Hand a file path or URI to the shell (default browser, mail client, …).
#[cfg(windows)]
pub fn shell_open(target: &str) -> Result<(), String> {
    windows_tool("explorer.exe")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Konnte {target} nicht öffnen: {e}"))
}
