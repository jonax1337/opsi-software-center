//! "Diagnosepaket für den Support": collects what a supporter asks for first
//! into one ZIP on the user's desktop, ready to attach to a ticket. Runs as the
//! user (no elevation): systeminfo, ipconfig /all, the user-scope gpresult
//! report and the opsi client logs where the user may read them.
//! `inhalt.txt` in the ZIP lists what made it in and why anything is missing.

#[cfg(windows)]
use std::path::{Path, PathBuf};

#[tauri::command]
pub async fn device_diagnostics_bundle() -> Result<String, String> {
    #[cfg(windows)]
    {
        crate::winproc::blocking(create_bundle).await?
    }
    #[cfg(not(windows))]
    {
        Err("nur unter Windows verfügbar".to_string())
    }
}

#[cfg(windows)]
const CONTENTS_FILE: &str = "inhalt.txt";
/// Below `%SystemDrive%\opsi.org\log`; often readable only by administrators.
#[cfg(windows)]
const OPSI_LOGS: [&str; 2] = ["opsiclientd.log", "opsi-script.log"];

/// Builds the ZIP and returns its path.
#[cfg(windows)]
fn create_bundle() -> Result<String, String> {
    let stamp = local_timestamp();
    let host = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into());
    let staging = std::env::temp_dir().join(format!("opsi-kiosk-diagnose-{stamp}"));
    std::fs::create_dir_all(&staging).map_err(|e| format!("Arbeitsordner nicht anlegbar: {e}"))?;

    let zip = collect_into(&staging, &format!("{host}, {stamp}"))
        .and_then(|()| crate::winproc::known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_Desktop))
        .map(|desktop| desktop.join(format!("Diagnose-{host}-{stamp}.zip")))
        .and_then(|zip| zip_folder(&staging, &zip).map(|()| zip));
    // leftovers in %TEMP% are harmless; the temp cleanup removes them later
    let _ = std::fs::remove_dir_all(&staging);
    zip.map(|path| path.to_string_lossy().into_owned())
}

/// Writes every diagnostic file it can into `dir`, plus the contents list.
#[cfg(windows)]
fn collect_into(dir: &Path, title: &str) -> Result<(), String> {
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let log_dir = PathBuf::from(format!(r"{system_drive}\opsi.org\log"));
    let mut steps = vec![
        ("systeminfo.txt", save_tool_output(dir, "systeminfo.txt", r"System32\systeminfo.exe", &["/fo", "list"])),
        ("ipconfig.txt", save_tool_output(dir, "ipconfig.txt", r"System32\ipconfig.exe", &["/all"])),
        ("gpresult.html", crate::device::write_gpresult_report(&dir.join("gpresult.html"))),
    ];
    for log in OPSI_LOGS {
        let copied = std::fs::copy(log_dir.join(log), dir.join(log)).map(drop).map_err(|e| e.to_string());
        steps.push((log, copied));
    }

    let mut contents = vec![format!("Diagnosepaket {title}"), String::new()];
    contents.extend(steps.into_iter().map(|(file, result)| match result {
        Ok(()) => format!("{file}: enthalten"),
        Err(reason) => format!("{file}: fehlt ({reason})"),
    }));
    std::fs::write(dir.join(CONTENTS_FILE), contents.join("\r\n"))
        .map_err(|e| format!("{CONTENTS_FILE} nicht schreibbar: {e}"))
}

/// Runs a Windows tool and saves its console output as UTF-8 text.
#[cfg(windows)]
fn save_tool_output(dir: &Path, file: &str, tool: &str, args: &[&str]) -> Result<(), String> {
    let output = crate::winproc::windows_tool(tool)
        .args(args)
        .output()
        .map_err(|e| format!("Start fehlgeschlagen: {e}"))?;
    if !output.status.success() {
        return Err(format!("beendet mit {}", output.status));
    }
    std::fs::write(dir.join(file), crate::winproc::console_text(&output.stdout)).map_err(|e| e.to_string())
}

/// Windows' own bsdtar writes ZIPs when the archive name ends in `.zip`.
#[cfg(windows)]
fn zip_folder(dir: &Path, zip: &Path) -> Result<(), String> {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("Arbeitsordner nicht lesbar: {e}"))?
        .flatten()
        .map(|entry| entry.file_name())
        .collect();
    let status = crate::winproc::windows_tool(r"System32\tar.exe")
        .args(["-a", "-c", "-f"])
        .arg(zip)
        .arg("-C")
        .arg(dir)
        .args(entries)
        .status()
        .map_err(|e| format!("tar konnte nicht gestartet werden: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ZIP-Erstellung fehlgeschlagen ({status})"))
    }
}

/// `yyyyMMdd-HHmm` in local time.
#[cfg(windows)]
fn local_timestamp() -> String {
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;

    // SAFETY: GetLocalTime only fills the SYSTEMTIME it is handed.
    let t = unsafe {
        let mut t = std::mem::zeroed();
        GetLocalTime(&mut t);
        t
    };
    format!("{:04}{:02}{:02}-{:02}{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute)
}
