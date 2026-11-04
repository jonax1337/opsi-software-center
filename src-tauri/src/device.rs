//! Device-level sync helpers (Company-Portal-style "Sync" extras).
//! Standard-user safe by design: gpupdate delegates the actual refresh to the
//! Group Policy Client service (runs as SYSTEM), the reboot check only reads
//! HKLM. Commands are hard-coded — no user input ever reaches a shell.

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

/// Generate an HTML RSOP report for the current user (`gpresult /h … /scope
/// user` — user scope avoids the elevation a computer-scope report needs) and
/// open it in the default browser. Returns the report path.
#[tauri::command]
pub async fn device_gpresult_report() -> Result<String, String> {
    #[cfg(windows)]
    {
        blocking(|| {
            let path = std::env::temp_dir().join("opsi-kiosk-gpresult.html");
            let path = path.to_string_lossy().to_string();
            let status = windows_tool(r"System32\gpresult.exe")
                .args(["/h", &path, "/f", "/scope", "user"])
                .status()
                .map_err(|e| format!("gpresult konnte nicht gestartet werden: {e}"))?;
            if !status.success() {
                return Err(format!("gpresult fehlgeschlagen ({status})"));
            }
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
