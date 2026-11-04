//! In-memory demo catalog that stands in for opsiclientd during development
//! and demos. Records have the exact shape of `getKioskProductInfosForClient`.
//! Processing is simulated: pending requests complete after a short delay.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::kiosk::ActionRequest;

const SIMULATED_PROCESSING: Duration = Duration::from_secs(4);

pub struct MockCatalog {
    products: Arc<Mutex<Vec<Value>>>,
}

impl MockCatalog {
    pub fn new() -> Self {
        Self {
            products: Arc::new(Mutex::new(demo_products())),
        }
    }

    pub async fn products(&self) -> Vec<Value> {
        self.products.lock().await.clone()
    }

    pub async fn set_action(&self, product_id: &str, action: ActionRequest) {
        let mut products = self.products.lock().await;
        if let Some(p) = products.iter_mut().find(|p| product_id_of(p) == product_id) {
            p["actionRequest"] = json!(action.as_str());
        }
    }

    /// Completes the pending requests of `product_ids` (all when `None`) in
    /// the background, like opsi's action processor would.
    pub fn process(&self, product_ids: Option<Vec<String>>) {
        let products = Arc::clone(&self.products);
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(SIMULATED_PROCESSING).await;
            let mut products = products.lock().await;
            for p in products.iter_mut() {
                let selected = product_ids
                    .as_ref()
                    .is_none_or(|ids| ids.iter().any(|id| id == product_id_of(p)));
                if selected {
                    complete_request(p);
                }
            }
        });
    }
}

/// Monogram icon (label, background) per demo product — neutral stand-ins,
/// no vendor logos, so the tile grid looks realistic without icon files.
const DEMO_ICONS: [(&str, &str, &str); 15] = [
    ("7zip", "7z", "#1F2937"),
    ("firefox-esr", "Fx", "#E66000"),
    ("vlc", "VLC", "#F28C00"),
    ("keepassxc", "Kx", "#3C8D2F"),
    ("greenshot", "Gs", "#2E7D32"),
    ("notepadplusplus", "N++", "#5A9E2F"),
    ("pdf24", "P24", "#C62828"),
    ("gimp", "Gi", "#5C5543"),
    ("libreoffice", "LO", "#18A303"),
    ("inkscape", "Ik", "#212121"),
    ("putty", "Pu", "#455A64"),
    ("winscp", "WS", "#1565C0"),
    ("vscode", "VS", "#0065A9"),
    ("teamviewer-qs", "TV", "#0E8EE9"),
    ("adobe-reader", "Ar", "#B30B00"),
];

/// Data-URL of the demo icon for `product_id`, if it is a demo product.
pub fn demo_icon(product_id: &str) -> Option<String> {
    use base64::Engine;

    let (_, label, color) = DEMO_ICONS.iter().find(|(id, _, _)| *id == product_id)?;
    let font_size = if label.len() > 2 { 20 } else { 26 };
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="14" fill="{color}"/><text x="32" y="32" dy=".35em" text-anchor="middle" font-family="Segoe UI Variable, Segoe UI, sans-serif" font-size="{font_size}" font-weight="600" fill="#fff">{label}</text></svg>"##
    );
    let b64 = base64::engine::general_purpose::STANDARD.encode(svg);
    Some(format!("data:image/svg+xml;base64,{b64}"))
}

fn product_id_of(p: &Value) -> &str {
    p.get("productId").and_then(Value::as_str).unwrap_or("")
}

/// Applies a pending setup/uninstall to the record and clears the request.
fn complete_request(p: &mut Value) {
    match p.get("actionRequest").and_then(Value::as_str) {
        Some("setup") => {
            p["installationStatus"] = json!("installed");
            p["installedVerStr"] = p["versionStr"].clone();
            p["updatePossible"] = json!(false);
            p["possibleAction"] = json!("uninstall");
        }
        Some("uninstall") => {
            p["installationStatus"] = json!("not_installed");
            p["installedVerStr"] = json!("");
            p["possibleAction"] = json!("setup");
        }
        _ => return,
    }
    p["actionResult"] = json!("successful");
    p["actionRequest"] = json!("none");
}

fn demo_product(
    id: &str,
    name: &str,
    description: &str,
    version: &str,
    installed_version: Option<&str>,
) -> Value {
    let installed = installed_version.is_some();
    json!({
        "productId": id,
        "productName": name,
        "description": description,
        "advice": "",
        "priority": 0,
        "productType": "LocalbootProduct",
        "productVersion": version,
        "packageVersion": "1",
        "versionStr": format!("{version}-1"),
        "installedVerStr": installed_version.map(|v| format!("{v}-1")).unwrap_or_default(),
        "installedProdVer": installed_version.unwrap_or_default(),
        "installedPackVer": if installed { "1" } else { "" },
        "updatePossible": installed && installed_version != Some(version),
        "possibleAction": if installed { "uninstall" } else { "setup" },
        "installationStatus": if installed { "installed" } else { "not_installed" },
        "actionRequest": "none",
        "actionResult": "",
        "hasSetup": true,
        "hasUninstall": true,
        "requirements": [],
    })
}

/// Enough products to exercise every UI state: updates, installed, available,
/// more than one home-page preview row, a failed last action, dependencies
/// and a product without description.
fn demo_products() -> Vec<Value> {
    let mut products = vec![
        demo_product("7zip", "7-Zip", "Packprogramm für ZIP-, 7z- und weitere Archivformate.", "24.09", Some("23.01")),
        demo_product("firefox-esr", "Mozilla Firefox ESR", "Webbrowser in der Extended-Support-Variante für Unternehmen.", "128.11.0", Some("128.11.0")),
        demo_product("vlc", "VLC Media Player", "Medienwiedergabe für nahezu alle Audio- und Videoformate.", "3.0.21", None),
        demo_product("keepassxc", "KeePassXC", "Lokaler Passwort-Manager mit KeePass-kompatiblem Datenbankformat.", "2.7.10", None),
        demo_product("greenshot", "Greenshot", "Screenshots erstellen, kommentieren und weitergeben.", "1.3.300", Some("1.3.300")),
        demo_product("notepadplusplus", "Notepad++", "Texteditor mit Syntax-Hervorhebung für viele Sprachen.", "8.8.1", None),
        demo_product("pdf24", "PDF24 Creator", "PDF-Dateien erstellen, zusammenführen und konvertieren.", "11.26.0", None),
        demo_product("gimp", "GIMP", "Bildbearbeitung mit Ebenen, Masken und zahlreichen Filtern.", "3.0.4", None),
        demo_product("libreoffice", "LibreOffice", "Office-Paket mit Textverarbeitung, Tabellenkalkulation und Präsentationen.\n\nÖffnet und speichert auch Microsoft-Office-Formate.", "25.2.4", Some("24.8.7")),
        demo_product("inkscape", "Inkscape", "Vektorgrafiken zeichnen und als SVG bearbeiten.", "1.4.2", None),
        demo_product("putty", "PuTTY", "SSH- und Telnet-Client für Remote-Verbindungen.", "0.83", Some("0.83")),
        demo_product("winscp", "WinSCP", "Dateien per SFTP, SCP und FTP übertragen.", "6.5.1", None),
        demo_product("vscode", "Visual Studio Code", "Code-Editor mit Erweiterungen, Git-Integration und Terminal.", "1.101.0", Some("1.99.3")),
        demo_product("teamviewer-qs", "TeamViewer QuickSupport", "", "15.66.5", None),
        demo_product("adobe-reader", "Adobe Acrobat Reader", "PDF-Dokumente anzeigen, drucken und kommentieren.", "25.001.20531", Some("25.001.20531")),
    ];
    for p in &mut products {
        match product_id_of(p) {
            "gimp" => p["actionResult"] = json!("failed"),
            "winscp" => {
                p["requirements"] = json!([{
                    "requiredProductId": "putty",
                    "required": "installed",
                    "preRequired": "",
                    "postRequired": "",
                }])
            }
            _ => {}
        }
    }
    products
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(installed: Option<&str>, action: &str) -> Value {
        let mut p = demo_product("x", "X", "", "2.0", installed);
        p["actionRequest"] = json!(action);
        p
    }

    #[test]
    fn setup_installs_the_offered_version() {
        let mut p = pending(Some("1.0"), "setup");
        complete_request(&mut p);
        assert_eq!(p["installationStatus"], "installed");
        assert_eq!(p["installedVerStr"], "2.0-1");
        assert_eq!(p["updatePossible"], false);
        assert_eq!(p["actionRequest"], "none");
        assert_eq!(p["actionResult"], "successful");
    }

    #[test]
    fn uninstall_removes_the_product() {
        let mut p = pending(Some("2.0"), "uninstall");
        complete_request(&mut p);
        assert_eq!(p["installationStatus"], "not_installed");
        assert_eq!(p["installedVerStr"], "");
        assert_eq!(p["possibleAction"], "setup");
        assert_eq!(p["actionRequest"], "none");
    }

    #[test]
    fn no_request_leaves_the_record_untouched() {
        let mut p = pending(None, "none");
        let before = p.clone();
        complete_request(&mut p);
        assert_eq!(p, before);
    }

    #[test]
    fn every_demo_product_has_a_demo_icon() {
        for p in demo_products() {
            let id = product_id_of(&p);
            let icon = demo_icon(id).unwrap_or_else(|| panic!("no demo icon for {id}"));
            assert!(icon.starts_with("data:image/svg+xml;base64,"));
        }
        assert!(demo_icon("not-a-demo-product").is_none());
    }

    #[test]
    fn demo_product_flags_outdated_installs_as_updatable() {
        assert_eq!(demo_product("a", "A", "", "2", Some("1"))["updatePossible"], true);
        assert_eq!(demo_product("a", "A", "", "2", Some("2"))["updatePossible"], false);
        assert_eq!(demo_product("a", "A", "", "2", None)["updatePossible"], false);
    }
}
