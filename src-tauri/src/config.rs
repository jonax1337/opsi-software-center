//! Per-organisation settings written by the opsi package, so one build serves
//! everyone: company name, IT-support contact, featured apps and the company
//! color, shown like Company Portal's branding. The opsi setup writes the product
//! properties to `HKLM\SOFTWARE\opsi-kiosk\Branding` (REG_SZ `company_name`,
//! `support_phone`, `support_email`, `support_url`, `featured_products` =
//! comma-separated product ids, `brand_color` = `#rrggbb`) — HKLM, because standard users cannot write there, so nobody can plant a
//! fake "IT support" link for other users.

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
    /// Product ids for the home-page "Ausgewählte Apps" banner, in banner order.
    pub featured_products: Vec<String>,
    /// Company color for buttons, selection and title bar as `#rrggbb`
    /// (lowercase); empty when unset or invalid — the app keeps its own blue.
    pub brand_color: String,
}

/// Missing key or values = no branding; the UI then shows neutral texts and no
/// featured banner. Demo mode features a few demo products unless set.
#[tauri::command]
pub async fn get_branding(state: tauri::State<'_, crate::kiosk::KioskState>) -> Result<Branding, String> {
    let mut branding = read_branding();
    if branding.featured_products.is_empty() && state.is_mock().await {
        branding.featured_products = crate::mock::DEMO_FEATURED.iter().map(|id| id.to_string()).collect();
    }
    Ok(branding)
}

/// "7zip, firefox-esr,,vlc" → ["7zip", "firefox-esr", "vlc"] (opsi-script joins
/// the multi-value property with commas).
fn product_list(value: &str) -> Vec<String> {
    value.split(',').map(str::trim).filter(|id| !id.is_empty()).map(String::from).collect()
}

/// "#E30613", "e30613" → "#e30613"; anything else → "" so a typo in the
/// property falls back to the default color instead of breaking the theme.
fn brand_color(value: &str) -> String {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("#{}", hex.to_ascii_lowercase())
    } else {
        String::new()
    }
}

fn read_branding() -> Branding {
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
            featured_products: product_list(&value("featured_products")),
            brand_color: brand_color(&value("brand_color")),
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
    let uri = support_uri(&read_branding(), channel)
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
            featured_products: Vec::new(),
            brand_color: String::new(),
        }
    }

    #[test]
    fn normalizes_the_brand_color() {
        assert_eq!(brand_color("#E30613"), "#e30613");
        assert_eq!(brand_color(" e30613 "), "#e30613");
    }

    #[test]
    fn rejects_brand_colors_that_are_not_six_hex_digits() {
        for invalid in ["", "#fff", "red", "#e3061", "#e306133", "#e3061g", "rgb(1,2,3)"] {
            assert_eq!(brand_color(invalid), "", "{invalid:?} must be rejected");
        }
    }

    #[test]
    fn splits_the_featured_products_list() {
        assert_eq!(product_list(" 7zip, firefox-esr,,vlc "), ["7zip", "firefox-esr", "vlc"]);
        assert!(product_list("").is_empty());
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
