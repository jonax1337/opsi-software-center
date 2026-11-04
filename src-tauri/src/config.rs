//! Per-organisation settings written by the opsi package (product properties
//! → `%ProgramData%\opsi-kiosk\branding.ini`), so one build serves everyone:
//! company name and IT-support contact, shown like Company Portal's branding.
//!
//! ```ini
//! [branding]
//! company_name=Example Ltd
//! support_phone=+49 30 1234567
//! support_email=it@example.org
//! support_url=https://helpdesk.example.org
//! ```

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DATA_DIR_NAME: &str = "opsi-kiosk";
const BRANDING_FILE: &str = "branding.ini";

/// `%ProgramData%\opsi-kiosk` — machine-wide, writable by the opsi package
/// (SYSTEM), readable by every user.
pub fn data_dir() -> Option<PathBuf> {
    std::env::var("ProgramData")
        .ok()
        .map(|dir| PathBuf::from(dir).join(DATA_DIR_NAME))
}

#[derive(Serialize, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Branding {
    pub company_name: String,
    pub support_phone: String,
    pub support_email: String,
    pub support_url: String,
}

/// Missing or unreadable file = no branding; the UI then shows neutral texts.
#[tauri::command]
pub fn get_branding() -> Branding {
    data_dir()
        .and_then(|dir| std::fs::read(dir.join(BRANDING_FILE)).ok())
        .map(|bytes| parse_branding(&decode(&bytes)))
        .unwrap_or_default()
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportChannel {
    Email,
    Phone,
    Website,
}

/// Opens the configured contact in the default handler. The frontend only
/// names the channel — the target always comes from the admin's config.
#[tauri::command]
pub fn open_support_contact(channel: SupportChannel) -> Result<(), String> {
    let uri = support_uri(&get_branding(), channel)
        .ok_or("Für diesen Kontaktweg ist nichts hinterlegt.")?;
    #[cfg(windows)]
    {
        crate::winproc::shell_open(&uri)
    }
    #[cfg(not(windows))]
    {
        Err(format!("nur unter Windows verfügbar ({uri})"))
    }
}

fn support_uri(branding: &Branding, channel: SupportChannel) -> Option<String> {
    let uri = match channel {
        SupportChannel::Email => {
            let email = branding.support_email.trim();
            email.contains('@').then(|| format!("mailto:{email}"))?
        }
        SupportChannel::Phone => {
            let digits: String = branding
                .support_phone
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '+')
                .collect();
            (!digits.is_empty()).then(|| format!("tel:{digits}"))?
        }
        SupportChannel::Website => {
            let url = branding.support_url.trim();
            (url.starts_with("https://") || url.starts_with("http://")).then(|| url.to_string())?
        }
    };
    is_shell_safe(&uri).then_some(uri)
}

/// No whitespace, quotes or control characters — the URI goes to explorer.exe.
fn is_shell_safe(uri: &str) -> bool {
    !uri.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"')
}

/// opsi-script may write the file as UTF-8 (with or without BOM) or in the
/// ANSI code page; Latin-1 decoding keeps German umlauts intact for the latter.
fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|&b| char::from(b)).collect(),
    }
}

/// `key=value` lines; sections, comments (`;`, `#`) and unknown keys are ignored.
fn parse_branding(text: &str) -> Branding {
    let mut branding = Branding::default();
    for line in text.lines().map(str::trim) {
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim().to_string();
        match key.trim() {
            "company_name" => branding.company_name = value,
            "support_phone" => branding.support_phone = value,
            "support_email" => branding.support_email = value,
            "support_url" => branding.support_url = value,
            _ => {}
        }
    }
    branding
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branding() -> Branding {
        Branding {
            company_name: "Beispiel GmbH".into(),
            support_phone: "+49 (30) 123-4567".into(),
            support_email: "it@example.org".into(),
            support_url: "https://helpdesk.example.org".into(),
        }
    }

    #[test]
    fn parses_known_keys_and_ignores_the_rest() {
        let parsed = parse_branding(
            "[branding]\n; comment\ncompany_name = Beispiel GmbH \nunknown=x\nsupport_email=it@example.org\n",
        );
        assert_eq!(parsed.company_name, "Beispiel GmbH");
        assert_eq!(parsed.support_email, "it@example.org");
        assert_eq!(parsed.support_phone, "");
    }

    #[test]
    fn keeps_equals_signs_inside_values() {
        let parsed = parse_branding("support_url=https://x.example/?a=b");
        assert_eq!(parsed.support_url, "https://x.example/?a=b");
    }

    #[test]
    fn decodes_utf8_with_bom_and_ansi_umlauts() {
        assert_eq!(decode(b"\xEF\xBB\xBFM\xC3\xBCller"), "Müller");
        assert_eq!(decode(b"M\xFCller"), "Müller");
    }

    #[test]
    fn builds_contact_uris() {
        let b = branding();
        assert_eq!(support_uri(&b, SupportChannel::Email).unwrap(), "mailto:it@example.org");
        assert_eq!(support_uri(&b, SupportChannel::Phone).unwrap(), "tel:+49301234567");
        assert_eq!(
            support_uri(&b, SupportChannel::Website).unwrap(),
            "https://helpdesk.example.org"
        );
    }

    #[test]
    fn rejects_missing_or_unsafe_contacts() {
        let unsafe_branding = Branding {
            support_email: "not-an-address".into(),
            support_url: "file:///C:/Windows/System32/cmd.exe".into(),
            ..Branding::default()
        };
        assert!(support_uri(&unsafe_branding, SupportChannel::Email).is_none());
        assert!(support_uri(&unsafe_branding, SupportChannel::Phone).is_none());
        assert!(support_uri(&unsafe_branding, SupportChannel::Website).is_none());

        let quoted = Branding {
            support_url: "https://x.example/\" /c calc".into(),
            ..Branding::default()
        };
        assert!(support_uri(&quoted, SupportChannel::Website).is_none());
    }
}
