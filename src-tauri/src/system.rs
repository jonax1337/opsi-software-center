//! System theming helpers. Reads the user's Windows accent color so the app's
//! `--primary`/`--ring` can match the native Fluent accent at runtime.
//! Read-only (HKCU), no user input ever reaches a shell.

use serde::Serialize;

/// The shades Windows itself derives from the user's accent color, lightest to
/// darkest. Windows uses Dark1 for accent fills on light backgrounds and
/// Light2/Light3 for accent on dark backgrounds — the frontend picks per theme.
/// All values are `#rrggbb`. `None` when the accent can't be read.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccentColor {
    pub base: Option<String>,
    pub light1: Option<String>,
    pub light2: Option<String>,
    pub light3: Option<String>,
    pub dark1: Option<String>,
    pub dark2: Option<String>,
    pub dark3: Option<String>,
}

#[cfg(windows)]
fn hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Read the user's accent palette from the registry.
///
/// `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent\AccentPalette`
/// is a 32-byte blob of 8 RGBA colors ordered light→dark:
/// Light3, Light2, Light1, Base, Dark1, Dark2, Dark3, (complement).
/// Falls back to the `HKCU\...\DWM\AccentColor` DWORD (ABGR) for `base` only.
#[tauri::command]
pub fn get_accent_color() -> AccentColor {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // Preferred: the full palette Windows computes itself.
        if let Ok(accent) =
            hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent")
        {
            if let Ok(val) = accent.get_raw_value("AccentPalette") {
                let b = &val.bytes;
                if b.len() >= 28 {
                    let c = |i: usize| hex(b[i], b[i + 1], b[i + 2]);
                    return AccentColor {
                        light3: Some(c(0)),
                        light2: Some(c(4)),
                        light1: Some(c(8)),
                        base: Some(c(12)),
                        dark1: Some(c(16)),
                        dark2: Some(c(20)),
                        dark3: Some(c(24)),
                    };
                }
            }
        }

        // Fallback: single DWM accent color, stored as 0x00BBGGRR (ABGR).
        if let Ok(dwm) = hkcu.open_subkey(r"Software\Microsoft\Windows\DWM") {
            if let Ok(v) = dwm.get_value::<u32, _>("AccentColor") {
                let r = (v & 0xFF) as u8;
                let g = ((v >> 8) & 0xFF) as u8;
                let b = ((v >> 16) & 0xFF) as u8;
                return AccentColor {
                    base: Some(hex(r, g, b)),
                    ..Default::default()
                };
            }
        }

        AccentColor::default()
    }
    #[cfg(not(windows))]
    {
        AccentColor::default()
    }
}
