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
fn run_gpupdate(target: &str, force: bool) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let mut args = vec![format!("/target:{target}"), "/wait:120".to_string()];
    if force {
        args.push("/force".to_string());
    }
    std::process::Command::new(format!(r"{windir}\System32\gpupdate.exe"))
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Refresh group policy for both scopes. `force` re-applies *all* policies
/// (the support-tool button); the automatic post-sync refresh passes `false`,
/// re-applying only changed policies — fast and side-effect free.
#[tauri::command]
pub async fn device_gpupdate(force: bool) -> Result<GpUpdateResult, String> {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || GpUpdateResult {
            computer: run_gpupdate("computer", force),
            user: run_gpupdate("user", force),
        })
        .await
        .map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = force;
        Ok(GpUpdateResult {
            computer: true,
            user: true,
        })
    }
}

/// Flush the DNS resolver cache (`ipconfig /flushdns`). Standard-user-safe;
/// a common first step for name-resolution / connectivity issues.
#[tauri::command]
pub async fn device_flush_dns() -> Result<bool, String> {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(|| {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
            std::process::Command::new(format!(r"{windir}\System32\ipconfig.exe"))
                .arg("/flushdns")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map(|o| o.status.success())
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
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
        tauri::async_runtime::spawn_blocking(|| {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
            let mut path = std::env::temp_dir();
            path.push("opsi-kiosk-gpresult.html");
            let p = path.to_string_lossy().to_string();
            let ok = std::process::Command::new(format!(r"{windir}\System32\gpresult.exe"))
                .args(["/h", &p, "/f", "/scope", "user"])
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .map(|s| s.success())
                .map_err(|e| e.to_string())?;
            if !ok {
                return Err("gpresult fehlgeschlagen".to_string());
            }
            // Open in the default browser (explorer routes .html to it).
            let _ = std::process::Command::new(format!(r"{windir}\explorer.exe"))
                .arg(&p)
                .spawn();
            Ok(p)
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        Err("nur unter Windows verfügbar".to_string())
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
