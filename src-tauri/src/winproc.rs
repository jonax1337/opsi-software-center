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
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    hidden_command(std::path::Path::new(&windir).join(relative_path))
}

/// `Command::new(program)` that opens no console window.
#[cfg(windows)]
pub fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut command = std::process::Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// A shell known folder of the signed-in user (`FOLDERID_Desktop`, …) — the
/// real location also when it is redirected, and unaffected by environment
/// variables the user can change.
#[cfg(windows)]
pub fn known_folder(id: &windows_sys::core::GUID) -> Result<std::path::PathBuf, String> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;

    let mut raw: *mut u16 = std::ptr::null_mut();
    // SAFETY: valid GUID and out pointer; the returned buffer is freed below
    // (the API requires that even on failure).
    let hr = unsafe { SHGetKnownFolderPath(id, 0, std::ptr::null_mut(), &mut raw) };
    let path = (hr >= 0).then(|| {
        // SAFETY: on success `raw` is a NUL-terminated UTF-16 string.
        unsafe {
            let len = (0..).take_while(|&i| *raw.add(i) != 0).count();
            std::path::PathBuf::from(std::ffi::OsString::from_wide(std::slice::from_raw_parts(raw, len)))
        }
    });
    // SAFETY: `raw` came from SHGetKnownFolderPath (or is null, which is allowed).
    unsafe { CoTaskMemFree(raw.cast()) };
    path.ok_or_else(|| format!("Systemordner nicht ermittelbar (HRESULT {hr:#x})"))
}

/// What a console tool printed into a pipe, as text: UTF-8 when it is valid
/// UTF-8, otherwise the OEM code page (cp850 on German Windows) that
/// ipconfig, systeminfo and Windows PowerShell use for piped output.
pub fn console_text(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    #[cfg(windows)]
    {
        oem_to_string(bytes)
    }
    #[cfg(not(windows))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

#[cfg(windows)]
fn oem_to_string(bytes: &[u8]) -> String {
    use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP};

    let Ok(len) = i32::try_from(bytes.len()) else {
        return String::from_utf8_lossy(bytes).into_owned();
    };
    // SAFETY: a null output buffer with size 0 only queries the needed length.
    let wide_len = unsafe { MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), len, std::ptr::null_mut(), 0) };
    let mut wide = vec![0u16; usize::try_from(wide_len).unwrap_or(0)];
    // SAFETY: `wide` has exactly the `wide_len` elements the call above asked for.
    let written = unsafe { MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), len, wide.as_mut_ptr(), wide_len) };
    wide.truncate(usize::try_from(written).unwrap_or(0));
    String::from_utf16_lossy(&wide)
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
