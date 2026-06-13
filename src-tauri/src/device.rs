//! Device-level sync helpers (Company-Portal-style "Sync" extras).
//! Standard-user safe by design: gpupdate delegates the actual refresh to the
//! Group Policy Client service (runs as SYSTEM), the reboot check only reads
//! HKLM. Commands are hard-coded — no user input ever reaches a shell.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpUpdateResult {
    pub computer: bool,
    pub user: bool,
}

#[cfg(windows)]
fn run_gpupdate(target: &str) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    std::process::Command::new(format!(r"{windir}\System32\gpupdate.exe"))
        .args([&format!("/target:{target}"), "/wait:120"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Refresh group policy for both scopes. Deliberately without `/force`:
/// only changed policies are re-applied, which keeps the call fast and
/// side-effect free.
#[tauri::command]
pub async fn device_gpupdate() -> Result<GpUpdateResult, String> {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(|| GpUpdateResult {
            computer: run_gpupdate("computer"),
            user: run_gpupdate("user"),
        })
        .await
        .map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        Ok(GpUpdateResult {
            computer: true,
            user: true,
        })
    }
}

/// True when Windows has a reboot pending (CBS, Windows Update or pending
/// file renames). opsi's own reboot prompt is suppressed fleet-wide, so the
/// footer indicator fed by this is the only place users see it.
#[tauri::command]
pub fn device_pending_reboot() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if hklm
            .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending")
            .is_ok()
        {
            return true;
        }
        if hklm
            .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired")
            .is_ok()
        {
            return true;
        }
        if let Ok(sm) = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager") {
            if sm.get_raw_value("PendingFileRenameOperations").is_ok() {
                return true;
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        false
    }
}
