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

/// Hand a file path or URI to its default handler (browser, mail client, …)
/// via `ShellExecuteW` — the target is passed as one string, never parsed as
/// a command line (unlike handing it to explorer.exe).
#[cfg(windows)]
pub fn shell_open(target: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    /// ShellExecuteW reports success with a value greater than 32.
    const SHELL_EXECUTE_OK: isize = 32;

    let wide = |s: &str| -> Vec<u16> { std::ffi::OsStr::new(s).encode_wide().chain([0]).collect() };
    let (verb, file) = (wide("open"), wide(target));
    // SAFETY: both strings are NUL-terminated UTF-16 buffers that outlive the call.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize > SHELL_EXECUTE_OK {
        Ok(())
    } else {
        Err(format!("Konnte {target} nicht öffnen (ShellExecute-Code {})", result as isize))
    }
}
