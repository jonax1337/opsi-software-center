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

fn demo_products() -> Vec<Value> {
    vec![
        demo_product("7zip", "7-Zip", "Packprogramm für ZIP-, 7z- und weitere Archivformate.", "24.09", Some("23.01")),
        demo_product("firefox-esr", "Mozilla Firefox ESR", "Webbrowser in der Extended-Support-Variante für Unternehmen.", "128.11.0", Some("128.11.0")),
        demo_product("vlc", "VLC Media Player", "Medienwiedergabe für nahezu alle Audio- und Videoformate.", "3.0.21", None),
        demo_product("keepassxc", "KeePassXC", "Lokaler Passwort-Manager mit KeePass-kompatiblem Datenbankformat.", "2.7.10", None),
        demo_product("greenshot", "Greenshot", "Screenshots erstellen, kommentieren und weitergeben.", "1.3.300", Some("1.3.300")),
        demo_product("notepadplusplus", "Notepad++", "Texteditor mit Syntax-Hervorhebung für viele Sprachen.", "8.8.1", None),
        demo_product("pdf24", "PDF24 Creator", "PDF-Dateien erstellen, zusammenführen und konvertieren.", "11.26.0", None),
    ]
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
    fn demo_product_flags_outdated_installs_as_updatable() {
        assert_eq!(demo_product("a", "A", "", "2", Some("1"))["updatePossible"], true);
        assert_eq!(demo_product("a", "A", "", "2", Some("2"))["updatePossible"], false);
        assert_eq!(demo_product("a", "A", "", "2", None)["updatePossible"], false);
    }
}
