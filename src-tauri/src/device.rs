//! Device-level sync and maintenance helpers (Company-Portal-style "Sync"
//! extras, the "Wartung" buttons). Standard-user safe by design: gpupdate
//! delegates the actual refresh to the Group Policy Client service (runs as
//! SYSTEM), the reboot check only reads HKLM, restart and Explorer restart only
//! touch the user's own session. Commands are hard-coded — no user input ever
//! reaches a shell.

use serde::Serialize;

use crate::winproc::blocking;
#[cfg(windows)]
use crate::winproc::{shell_open, windows_tool};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpUpdateResult {
    pub computer: bool,
    pub user: bool,
}

/// Refresh group policy for both scopes. `force` re-applies *all* policies
/// (the support-tool button); the automatic post-sync refresh passes `false`,
/// re-applying only changed policies — fast and side-effect free.
#[tauri::command]
pub async fn device_gpupdate(force: bool) -> Result<GpUpdateResult, String> {
    blocking(move || GpUpdateResult {
        computer: run_gpupdate("computer", force),
        user: run_gpupdate("user", force),
    })
    .await
}

#[cfg(windows)]
fn run_gpupdate(target: &str, force: bool) -> bool {
    let mut command = windows_tool(r"System32\gpupdate.exe");
    command.args([format!("/target:{target}"), "/wait:120".to_string()]);
    if force {
        command.arg("/force");
    }
    command.output().is_ok_and(|o| o.status.success())
}

#[cfg(not(windows))]
fn run_gpupdate(_target: &str, _force: bool) -> bool {
    true
}

/// Flush the DNS resolver cache (`ipconfig /flushdns`). Standard-user-safe;
/// a common first step for name-resolution / connectivity issues.
#[tauri::command]
pub async fn device_flush_dns() -> Result<bool, String> {
    #[cfg(windows)]
    {
        blocking(|| {
            windows_tool(r"System32\ipconfig.exe")
                .arg("/flushdns")
                .output()
                .map(|o| o.status.success())
                .map_err(|e| format!("ipconfig konnte nicht gestartet werden: {e}"))
        })
        .await?
    }
    #[cfg(not(windows))]
    {
        Ok(true)
    }
}

/// Generate an HTML RSOP report for the current user and open it in the
/// default browser. Returns the report path.
#[tauri::command]
pub async fn device_gpresult_report() -> Result<String, String> {
    #[cfg(windows)]
    {
        blocking(|| {
            let path = std::env::temp_dir().join("opsi-kiosk-gpresult.html");
            write_gpresult_report(&path)?;
            let path = path.to_string_lossy().to_string();
            shell_open(&path)?;
            Ok(path)
        })
        .await?
    }
    #[cfg(not(windows))]
    {
        Err("nur unter Windows verfügbar".to_string())
    }
}

/// `gpresult /h … /scope user` — user scope avoids the elevation a
/// computer-scope report needs. Overwrites `path`.
#[cfg(windows)]
pub fn write_gpresult_report(path: &std::path::Path) -> Result<(), String> {
    let status = windows_tool(r"System32\gpresult.exe")
        .arg("/h")
        .arg(path)
        .args(["/f", "/scope", "user"])
        .status()
        .map_err(|e| format!("gpresult konnte nicht gestartet werden: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("gpresult fehlgeschlagen ({status})"))
    }
}

/// Restart the computer now. Without `/f`, apps with unsaved work can still
/// veto it; the UI asks for confirmation first. Standard users hold the
/// shutdown privilege on workstations.
#[tauri::command]
pub async fn device_restart() -> Result<(), String> {
    #[cfg(windows)]
    {
        blocking(|| {
            let status = windows_tool(r"System32\shutdown.exe")
                .args(["/r", "/t", "0"])
                .status()
                .map_err(|e| format!("shutdown konnte nicht gestartet werden: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("Neustart abgelehnt ({status})"))
            }
        })
        .await?
    }
    #[cfg(not(windows))]
    {
        Err("nur unter Windows verfügbar".to_string())
    }
}

/// Restart Windows Explorer (taskbar, start menu, desktop) in the user's own
/// session — the classic fix for a frozen taskbar.
#[tauri::command]
pub async fn device_restart_explorer() -> Result<(), String> {
    #[cfg(windows)]
    {
        blocking(restart_explorer).await?
    }
    #[cfg(not(windows))]
    {
        Err("nur unter Windows verfügbar".to_string())
    }
}

#[cfg(windows)]
fn restart_explorer() -> Result<(), String> {
    use std::time::Duration;

    /// Winlogon normally restarts the shell by itself (AutoRestartShell);
    /// only when it has not within this window do we start it.
    const SHELL_RESTART_POLLS: u32 = 10;
    const POLL_INTERVAL: Duration = Duration::from_millis(500);

    let session_filter = format!("SESSION eq {}", current_session_id()?);
    windows_tool(r"System32\taskkill.exe")
        .args(["/f", "/im", "explorer.exe", "/fi", &session_filter])
        .output()
        .map_err(|e| format!("taskkill konnte nicht gestartet werden: {e}"))?;
    for _ in 0..SHELL_RESTART_POLLS {
        std::thread::sleep(POLL_INTERVAL);
        // a failed check counts as "not running": the taskbar must come back
        // either way, and a second shell instance only opens a window
        if explorer_running(&session_filter).unwrap_or(false) {
            return Ok(());
        }
    }
    windows_tool("explorer.exe")
        .spawn()
        .map(drop)
        .map_err(|e| format!("Explorer konnte nicht gestartet werden: {e}"))
}

#[cfg(windows)]
fn explorer_running(session_filter: &str) -> Result<bool, String> {
    let output = windows_tool(r"System32\tasklist.exe")
        .args(["/nh", "/fo", "csv", "/fi", "IMAGENAME eq explorer.exe", "/fi", session_filter])
        .output()
        .map_err(|e| format!("tasklist konnte nicht gestartet werden: {e}"))?;
    // CSV rows quote the image name; the "no tasks" notice is localized and unquoted
    Ok(String::from_utf8_lossy(&output.stdout)
        .to_ascii_lowercase()
        .contains("\"explorer.exe\""))
}

#[cfg(windows)]
fn current_session_id() -> Result<u32, String> {
    use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;

    let mut session = 0u32;
    // SAFETY: `session` is a valid, writable u32 for the duration of the call.
    if unsafe { ProcessIdToSessionId(std::process::id(), &mut session) } == 0 {
        return Err(format!("Sitzung nicht ermittelbar: {}", std::io::Error::last_os_error()));
    }
    Ok(session)
}

/// True when Windows has a reboot pending (CBS, Windows Update or pending
/// file renames). Shown in the UI because opsi's own reboot prompt is often
/// suppressed.
#[tauri::command]
pub fn device_pending_reboot() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;

        const REBOOT_FLAG_KEYS: [&str; 2] = [
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired",
        ];
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let flag_key_exists = REBOOT_FLAG_KEYS.iter().any(|k| hklm.open_subkey(k).is_ok());
        let renames_pending = hklm
            .open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager")
            .is_ok_and(|sm| sm.get_raw_value("PendingFileRenameOperations").is_ok());
        flag_key_exists || renames_pending
    }
    #[cfg(not(windows))]
    {
        false
    }
}
