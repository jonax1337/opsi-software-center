//! Admin-defined maintenance actions ("Aktionen Ihrer IT"): scripts the opsi
//! package ships to `actions\` in the install folder (`%ProgramFiles%\Software
//! Center\actions\`), listed in `actions.json` there, run as the signed-in user.
//!
//! Trust model: the folder sits next to the executable under Program Files,
//! where standard users can neither create nor change anything — unlike
//! %ProgramData%, where any user may create (and then own) a folder before the
//! package does. So manifest and scripts always come from the admin. The
//! frontend only names an action id; the script path is resolved here from the
//! manifest, never taken from the UI, and nothing the user types reaches it.
//!
//! Manifest — a JSON array:
//! ```json
//! [{ "id": "teams-cache", "label": "Teams-Cache leeren",
//!    "description": "Beendet Teams und leert den Cache.",
//!    "script": "teams-cache.ps1", "confirm": "Teams wird dabei beendet." }]
//! ```
//! `description` and `confirm` are optional. Scripts: `.ps1` (Windows
//! PowerShell, `-ExecutionPolicy Bypass` — a GPO execution policy still wins)
//! or `.cmd`/`.bat`. Exit code 0 = success; the last line the script prints
//! becomes the success message, the first error line the failure message.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::winproc::blocking;

const ACTIONS_DIR: &str = "actions";
const MANIFEST_FILE: &str = "actions.json";
/// A hung script must not block the button forever.
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MESSAGE_MAX_CHARS: usize = 300;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    /// Asked before running; empty = run on click.
    #[serde(default)]
    pub confirm: String,
    /// File name inside the actions folder — the UI never sees it.
    #[serde(skip_serializing)]
    script: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ScriptKind {
    PowerShell,
    Batch,
}

impl ScriptKind {
    fn of(file_name: &str) -> Option<Self> {
        let (_, ext) = file_name.rsplit_once('.')?;
        match ext.to_ascii_lowercase().as_str() {
            "ps1" => Some(Self::PowerShell),
            "cmd" | "bat" => Some(Self::Batch),
            _ => None,
        }
    }
}

/// The configured actions; none when the package ships no manifest. A broken
/// manifest is an error (shown to the user) rather than silently hidden, so the
/// admin notices it on the first test client.
#[tauri::command]
pub async fn admin_actions_list() -> Result<Vec<AdminAction>, String> {
    blocking(|| match actions_dir() {
        Some(dir) => load_actions(&dir),
        None => Ok(Vec::new()),
    })
    .await?
}

/// Runs the action's script and returns its message (last printed line).
#[tauri::command]
pub async fn admin_action_run(id: String) -> Result<Option<String>, String> {
    blocking(move || {
        let dir = actions_dir().ok_or("Kein Aktionsordner vorhanden.")?;
        let action = load_actions(&dir)?
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("Aktion „{id}“ ist nicht (mehr) eingerichtet."))?;
        run_script(&dir, &action.script)
    })
    .await?
}

/// Next to the running executable — not derived from environment variables.
fn actions_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join(ACTIONS_DIR))
}

fn load_actions(dir: &Path) -> Result<Vec<AdminAction>, String> {
    let manifest = dir.join(MANIFEST_FILE);
    let json = match std::fs::read_to_string(&manifest) {
        Ok(json) => json,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{MANIFEST_FILE} nicht lesbar: {e}")),
    };
    let actions = parse_manifest(&json)?;
    if let Some(missing) = actions.iter().find(|a| !dir.join(&a.script).is_file()) {
        return Err(format!("{MANIFEST_FILE}: Skript „{}“ fehlt im Aktionsordner.", missing.script));
    }
    Ok(actions)
}

fn parse_manifest(json: &str) -> Result<Vec<AdminAction>, String> {
    // a UTF-8 BOM (Notepad, PowerShell's Out-File) is not JSON
    let json = json.trim_start_matches('\u{feff}');
    let actions: Vec<AdminAction> =
        serde_json::from_str(json).map_err(|e| format!("{MANIFEST_FILE} ist ungültig: {e}"))?;
    for (index, action) in actions.iter().enumerate() {
        validate(action).map_err(|reason| format!("{MANIFEST_FILE}, Eintrag {}: {reason}", index + 1))?;
        if actions[..index].iter().any(|a| a.id == action.id) {
            return Err(format!("{MANIFEST_FILE}: id „{}“ ist doppelt.", action.id));
        }
    }
    Ok(actions)
}

fn validate(action: &AdminAction) -> Result<(), String> {
    let id_char = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    if action.id.is_empty() || !action.id.chars().all(id_char) {
        return Err(format!("id „{}“ darf nur aus a–z, 0–9, - und _ bestehen.", action.id));
    }
    if action.label.trim().is_empty() {
        return Err("label fehlt.".into());
    }
    // a bare file name: no separators, drive letters or "..", so the script
    // can only ever come from the admin-only actions folder
    let file_char = |c: char| c.is_ascii_alphanumeric() || "-_.".contains(c);
    if !action.script.chars().all(file_char) || action.script.starts_with('.') {
        return Err(format!("script „{}“ muss ein einfacher Dateiname sein.", action.script));
    }
    if ScriptKind::of(&action.script).is_none() {
        return Err(format!("script „{}“: erlaubt sind .ps1, .cmd und .bat.", action.script));
    }
    Ok(())
}

#[cfg(windows)]
fn run_script(dir: &Path, script: &str) -> Result<Option<String>, String> {
    use crate::winproc::{console_text, hidden_command, windows_tool};

    let path = dir.join(script);
    let mut command = match ScriptKind::of(script) {
        Some(ScriptKind::PowerShell) => {
            let mut c = windows_tool(r"System32\WindowsPowerShell\v1.0\powershell.exe");
            c.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]);
            c.arg(&path);
            c
        }
        // std runs batch files through cmd.exe itself (/d: no AutoRun) and
        // escapes the path for cmd's own parser — "Program Files" included
        Some(ScriptKind::Batch) => hidden_command(&path),
        None => return Err(format!("Skripttyp von „{script}“ wird nicht unterstützt.")),
    };
    command.current_dir(dir);

    let output = run_with_timeout(command, SCRIPT_TIMEOUT)?;
    let stdout = console_text(&output.stdout);
    if output.status.success() {
        return Ok(last_line(&stdout));
    }
    let reason = first_line(&console_text(&output.stderr)).or_else(|| last_line(&stdout));
    let code = output.status.code().map_or_else(|| "?".to_string(), |c| c.to_string());
    Err(match reason {
        Some(reason) => format!("Exit-Code {code}: {reason}"),
        None => format!("Exit-Code {code}"),
    })
}

#[cfg(not(windows))]
fn run_script(_dir: &Path, _script: &str) -> Result<Option<String>, String> {
    Err("nur unter Windows verfügbar".to_string())
}

/// `Command::output` with a deadline: the script is killed when it runs over.
/// Pipes are drained on their own threads so a chatty script cannot block on
/// a full pipe while we wait.
#[cfg(windows)]
fn run_with_timeout(mut command: std::process::Command, timeout: Duration) -> Result<std::process::Output, String> {
    use std::process::Stdio;
    use std::time::Instant;

    /// Background processes the script started may hold its pipes open.
    const PIPE_DRAIN: Duration = Duration::from_secs(2);
    const POLL_INTERVAL: Duration = Duration::from_millis(200);

    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Skript konnte nicht gestartet werden: {e}"))?;
    let stdout = read_in_background(child.stdout.take());
    let stderr = read_in_background(child.stderr.take());

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| format!("Skriptstatus unbekannt: {e}"))? {
            break status;
        }
        if Instant::now() >= deadline {
            // the timeout error below is what matters; a failed kill means it just ended
            let _ = child.kill();
            return Err(format!("Zeitüberschreitung: Skript nach {} Minuten abgebrochen.", timeout.as_secs() / 60));
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    let drain_deadline = Instant::now() + PIPE_DRAIN;
    Ok(std::process::Output {
        status,
        stdout: collect_until(&stdout, drain_deadline),
        stderr: collect_until(&stderr, drain_deadline),
    })
}

/// Forwards what the pipe delivers chunk by chunk, so the output arrives even
/// while a program the script started (`start`, `Start-Process`) still holds
/// the inherited pipe open and it never reaches end-of-file.
#[cfg(windows)]
fn read_in_background<R: std::io::Read + Send + 'static>(pipe: Option<R>) -> std::sync::mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = std::sync::mpsc::channel();
    if let Some(mut pipe) = pipe {
        std::thread::spawn(move || {
            let mut chunk = [0u8; 4096];
            // ends at end-of-file, on a read error or once nobody listens any more
            while let Ok(n @ 1..) = pipe.read(&mut chunk) {
                if tx.send(chunk[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
    }
    rx
}

/// Everything received until the pipe closes or `deadline` passes — plus what
/// is already waiting, so a pipe drained after the deadline (stderr, when
/// stdout used up the wait) still yields what the script wrote.
#[cfg(windows)]
fn collect_until(rx: &std::sync::mpsc::Receiver<Vec<u8>>, deadline: std::time::Instant) -> Vec<u8> {
    let mut out = Vec::new();
    while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(chunk) => out.extend(chunk),
            Err(_) => break,
        }
    }
    out.extend(rx.try_iter().flatten());
    out
}

fn last_line(text: &str) -> Option<String> {
    text.lines().map(str::trim).rfind(|l| !l.is_empty()).map(clip)
}

fn first_line(text: &str) -> Option<String> {
    text.lines().map(str::trim).find(|l| !l.is_empty()).map(clip)
}

fn clip(line: &str) -> String {
    line.chars().take(MESSAGE_MAX_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"[
        { "id": "teams-cache", "label": "Teams-Cache leeren", "script": "teams-cache.ps1",
          "description": "Beendet Teams.", "confirm": "Teams wird beendet." },
        { "id": "drive_map", "label": "Laufwerke verbinden", "script": "Map-Drives.CMD" }
    ]"#;

    #[test]
    fn parses_a_valid_manifest_with_optional_fields() {
        let actions = parse_manifest(VALID).unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].confirm, "Teams wird beendet.");
        assert_eq!(actions[1].description, "");
        assert_eq!(ScriptKind::of(&actions[1].script), Some(ScriptKind::Batch));
    }

    #[test]
    fn accepts_a_utf8_bom() {
        assert_eq!(parse_manifest(&format!("\u{feff}{VALID}")).unwrap().len(), 2);
    }

    #[test]
    fn never_exposes_the_script_to_the_ui() {
        let json = serde_json::to_value(&parse_manifest(VALID).unwrap()[0]).unwrap();
        assert!(json.get("script").is_none());
        assert_eq!(json["label"], "Teams-Cache leeren");
    }

    #[test]
    fn rejects_scripts_outside_the_actions_folder() {
        for script in [
            r"..\evil.ps1",
            "../evil.ps1",
            r"C:\Windows\evil.ps1",
            r"\\server\share\evil.ps1",
            "sub/evil.ps1",
            "..ps1",
            "with space.ps1",
        ] {
            let json = format!(r#"[{{ "id": "x", "label": "X", "script": {script:?} }}]"#);
            assert!(parse_manifest(&json).is_err(), "{script:?} must be rejected");
        }
    }

    #[test]
    fn rejects_unsupported_script_types() {
        for script in ["tool.exe", "macro.vbs", "noextension"] {
            let json = format!(r#"[{{ "id": "x", "label": "X", "script": "{script}" }}]"#);
            assert!(parse_manifest(&json).is_err(), "{script:?} must be rejected");
        }
    }

    #[test]
    fn rejects_bad_ids_labels_duplicates_and_typos() {
        for json in [
            r#"[{ "id": "", "label": "X", "script": "a.ps1" }]"#,
            r#"[{ "id": "a b", "label": "X", "script": "a.ps1" }]"#,
            r#"[{ "id": "a", "label": " ", "script": "a.ps1" }]"#,
            r#"[{ "id": "a", "label": "X", "script": "a.ps1" }, { "id": "a", "label": "Y", "script": "b.ps1" }]"#,
            r#"[{ "id": "a", "label": "X", "script": "a.ps1", "confrim": "?" }]"#,
            r#"{ "id": "a" }"#,
        ] {
            assert!(parse_manifest(json).is_err(), "{json} must be rejected");
        }
    }

    #[test]
    fn missing_manifest_means_no_actions() {
        let dir = std::env::temp_dir().join(format!("opsi-kiosk-actions-test-{}", std::process::id()));
        assert_eq!(load_actions(&dir).unwrap(), Vec::new());
    }

    #[test]
    fn reports_a_listed_script_that_is_missing() {
        let dir = std::env::temp_dir().join(format!("opsi-kiosk-actions-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(MANIFEST_FILE), r#"[{ "id": "a", "label": "A", "script": "a.ps1" }]"#).unwrap();

        let err = load_actions(&dir).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(err.contains("a.ps1"), "{err}");
    }

    #[cfg(windows)]
    mod run {
        use super::super::*;

        fn run_temp_script(name: &str, body: &[u8]) -> Result<Option<String>, String> {
            let dir = std::env::temp_dir().join(format!("opsi-kiosk-run-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), body).unwrap();
            let result = run_script(&dir, name);
            let _ = std::fs::remove_dir_all(&dir);
            result
        }

        #[test]
        fn batch_success_reports_the_last_line() {
            let result = run_temp_script("ok.cmd", b"@echo off\r\necho Schritt 1\r\necho Fertig\r\n");
            assert_eq!(result, Ok(Some("Fertig".into())));
        }

        #[test]
        fn batch_failure_reports_exit_code_and_error_line() {
            let result = run_temp_script("fail.cmd", b"@echo off\r\necho kaputt 1>&2\r\nexit /b 3\r\n");
            assert_eq!(result, Err("Exit-Code 3: kaputt".into()));
        }

        #[test]
        fn powershell_umlauts_survive_the_console_code_page() {
            let script = "\u{feff}Write-Output \"Größe: 5 MB\"\r\n";
            let result = run_temp_script("umlaut.ps1", script.as_bytes());
            assert_eq!(result, Ok(Some("Größe: 5 MB".into())));
        }

        #[test]
        fn powershell_exception_is_a_failure() {
            let result = run_temp_script("throw.ps1", b"throw 'nope'\r\n");
            let err = result.unwrap_err();
            assert!(err.starts_with("Exit-Code 1: "), "{err}");
            assert!(err.contains("nope"), "{err}");
        }

        #[test]
        fn a_background_program_holding_the_pipe_does_not_swallow_the_message() {
            let started = std::time::Instant::now();
            // ping's stdout goes to nul, but it inherits stderr and holds that pipe open
            let result = run_temp_script(
                "bg.cmd",
                b"@echo off\r\necho Gestartet\r\nstart \"\" /b ping -n 8 127.0.0.1 >nul\r\n",
            );
            assert_eq!(result, Ok(Some("Gestartet".into())));
            assert!(started.elapsed() < std::time::Duration::from_secs(6));
        }

        #[test]
        fn the_error_line_survives_a_background_program_holding_stdout() {
            // ping keeps writing to the inherited stdout pipe after the script exits
            let result = run_temp_script(
                "bg-fail.cmd",
                b"@echo off\r\nstart \"\" /b ping -n 8 127.0.0.1 2>nul\r\necho kaputt 1>&2\r\nexit /b 2\r\n",
            );
            assert_eq!(result, Err("Exit-Code 2: kaputt".into()));
        }
    }

    #[test]
    fn picks_message_lines() {
        let text = "\r\n  Schritt 1\r\nFertig: 120 MB frei  \r\n\r\n";
        assert_eq!(last_line(text).as_deref(), Some("Fertig: 120 MB frei"));
        assert_eq!(first_line(text).as_deref(), Some("Schritt 1"));
        assert_eq!(last_line(" \n "), None);
        assert_eq!(clip(&"x".repeat(500)).len(), MESSAGE_MAX_CHARS);
    }
}
