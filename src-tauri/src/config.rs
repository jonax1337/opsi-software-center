//! Per-organisation settings written by the opsi package, so one build serves
//! everyone: company name and IT-support contact, shown like Company Portal's
//! branding. The opsi setup writes the product properties to
//! `HKLM\SOFTWARE\opsi-kiosk\Branding` (REG_SZ `company_name`, `support_phone`,
//! `support_email`, `support_url`) — HKLM, because standard users cannot write
//! there, so nobody can plant a fake "IT support" link for other users.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DATA_DIR_NAME: &str = "opsi-kiosk";
#[cfg(windows)]
const BRANDING_KEY: &str = r"SOFTWARE\opsi-kiosk\Branding";

/// `%ProgramData%\opsi-kiosk` — machine-wide; the opsi package restricts
/// write access to SYSTEM/Administrators.
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

/// Missing key or values = no branding; the UI then shows neutral texts.
#[tauri::command]
pub fn get_branding() -> Branding {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;

        let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(BRANDING_KEY) else {
            return Branding::default();
        };
        let value = |name: &str| key.get_value::<String, _>(name).unwrap_or_default().trim().to_string();
        Branding {
            company_name: value("company_name"),
            support_phone: value("support_phone"),
            support_email: value("support_email"),
            support_url: value("support_url"),
        }
    }
    #[cfg(not(windows))]
    {
        Branding::default()
    }
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
    match channel {
        SupportChannel::Email => {
            let email = branding.support_email.trim();
            is_plain_email(email).then(|| format!("mailto:{email}"))
        }
        SupportChannel::Phone => {
            let digits: String = branding
                .support_phone
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '+')
                .collect();
            (!digits.is_empty()).then(|| format!("tel:{digits}"))
        }
        SupportChannel::Website => {
            let url = branding.support_url.trim();
            let has_web_scheme = url.starts_with("https://") || url.starts_with("http://");
            (has_web_scheme && is_plain_uri(url)).then(|| url.to_string())
        }
    }
}

/// One `@`, a dotted domain and only characters of an ordinary address — no
/// `?`/`&` (mailto header injection), separators or quotes.
fn is_plain_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else { return false };
    let allowed = |c: char| c.is_ascii_alphanumeric() || "._+-".contains(c);
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && local.chars().all(allowed)
        && domain.chars().all(|c| c.is_ascii_alphanumeric() || ".-".contains(c))
}

/// No whitespace, quotes or control characters.
fn is_plain_uri(uri: &str) -> bool {
    !uri.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branding() -> Branding {
        Branding {
            company_name: "Beispiel GmbH".into(),
            support_phone: "+49 (30) 123-4567".into(),
            support_email: "it@example.org".into(),
            support_url: "https://helpdesk.example.org/?q=a%20b".into(),
        }
    }

    #[test]
    fn builds_contact_uris() {
        let b = branding();
        assert_eq!(support_uri(&b, SupportChannel::Email).unwrap(), "mailto:it@example.org");
        assert_eq!(support_uri(&b, SupportChannel::Phone).unwrap(), "tel:+49301234567");
        assert_eq!(
            support_uri(&b, SupportChannel::Website).unwrap(),
            "https://helpdesk.example.org/?q=a%20b"
        );
    }

    #[test]
    fn rejects_unsafe_emails() {
        for email in [
            "not-an-address",
            "a@b@example.org",
            "it@example.org?subject=x&body=y",
            "it@example.org,evil@example.org",
            "it@localhost",
            r"it@exa\mple.org",
            "\"it\"@example.org",
        ] {
            let b = Branding { support_email: email.into(), ..Branding::default() };
            assert!(support_uri(&b, SupportChannel::Email).is_none(), "{email:?} must be rejected");
        }
    }

    #[test]
    fn rejects_missing_or_unsafe_contacts() {
        let unsafe_branding = Branding {
            support_url: "file:///C:/Windows/System32/cmd.exe".into(),
            ..Branding::default()
        };
        assert!(support_uri(&unsafe_branding, SupportChannel::Phone).is_none());
        assert!(support_uri(&unsafe_branding, SupportChannel::Website).is_none());

        let quoted = Branding {
            support_url: "https://x.example/\" /c calc".into(),
            ..Branding::default()
        };
        assert!(support_uri(&quoted, SupportChannel::Website).is_none());
    }
}
