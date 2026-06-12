//! JSON-RPC client for the opsiclientd kiosk endpoint (https://localhost:4441/kiosk).
//!
//! The endpoint allows unauthenticated access from localhost when
//! `control_server.kiosk_api_active` is enabled (opsi default). It exposes the
//! same interface the official opsi-client-kiosk uses; the relevant methods are
//! `getClientId`, `getKioskProductInfosForClient`,
//! `setProductActionRequestWithDependencies` and `processActionRequests`.
//!
//! When opsiclientd is not reachable (developer machine, demo), the backend
//! falls back to an in-memory mock catalog so the full UI flow stays testable.
//! Force the mock with env var `OPSI_KIOSK_MOCK=1`.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

const KIOSK_URL: &str = "https://localhost:4441/kiosk";

#[derive(Clone, PartialEq)]
enum Backend {
    Unknown,
    Live { client_id: String },
    Mock,
}

pub struct KioskState {
    http: reqwest::Client,
    rpc_id: AtomicU64,
    backend: Mutex<Backend>,
    mock_products: Arc<Mutex<Vec<Value>>>,
}

impl KioskState {
    pub fn new() -> Self {
        Self {
            // opsiclientd serves a self-signed certificate on localhost
            http: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("failed to build http client"),
            rpc_id: AtomicU64::new(1),
            backend: Mutex::new(Backend::Unknown),
            mock_products: Arc::new(Mutex::new(mock_catalog())),
        }
    }

    async fn rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.rpc_id.fetch_add(1, Ordering::Relaxed);
        let body = json!({ "id": id, "method": method, "params": params });
        let resp = self
            .http
            .post(KIOSK_URL)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("opsiclientd nicht erreichbar: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("opsiclientd antwortete mit HTTP {status}"));
        }
        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| format!("Ungültige JSON-RPC-Antwort: {e}"))?;
        if let Some(err) = parsed.get("error") {
            if !err.is_null() {
                let msg = err
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unbekannter Fehler");
                return Err(format!("opsi-Fehler bei {method}: {msg}"));
            }
        }
        Ok(parsed.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Resolve the backend once: live opsiclientd if reachable, mock otherwise.
    async fn resolve_backend(&self) -> Backend {
        let mut backend = self.backend.lock().await;
        if *backend != Backend::Unknown {
            return backend.clone();
        }
        if std::env::var("OPSI_KIOSK_MOCK").as_deref() == Ok("1") {
            *backend = Backend::Mock;
            return backend.clone();
        }
        match self.rpc("getClientId", json!([])).await {
            Ok(Value::String(client_id)) if !client_id.is_empty() => {
                *backend = Backend::Live { client_id };
            }
            _ => {
                *backend = Backend::Mock;
            }
        }
        backend.clone()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KioskStatus {
    connected: bool,
    mock: bool,
    client_id: String,
    depot_id: String,
    version: String,
}

#[tauri::command]
pub async fn kiosk_status(state: tauri::State<'_, KioskState>) -> Result<KioskStatus, String> {
    match state.resolve_backend().await {
        Backend::Live { client_id } => {
            let config = state
                .rpc("getConfigDataFromOpsiclientd", json!([true, false]))
                .await
                .unwrap_or(Value::Null);
            Ok(KioskStatus {
                connected: true,
                mock: false,
                client_id,
                depot_id: config
                    .get("depot_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                version: config
                    .get("opsiclientd_version")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        }
        _ => Ok(KioskStatus {
            connected: false,
            mock: true,
            client_id: "demo-client.example.org".into(),
            depot_id: "opsi-depot.example.org".into(),
            version: "mock".into(),
        }),
    }
}

#[tauri::command]
pub async fn kiosk_get_products(state: tauri::State<'_, KioskState>) -> Result<Vec<Value>, String> {
    match state.resolve_backend().await {
        Backend::Live { client_id } => {
            let result = state
                .rpc("getKioskProductInfosForClient", json!([client_id, false]))
                .await?;
            match result {
                Value::Array(products) => Ok(products),
                other => Err(format!("Unerwartetes Antwortformat: {other}")),
            }
        }
        _ => Ok(state.mock_products.lock().await.clone()),
    }
}

#[tauri::command]
pub async fn kiosk_set_action(
    state: tauri::State<'_, KioskState>,
    product_id: String,
    action: String,
) -> Result<(), String> {
    if !matches!(action.as_str(), "setup" | "uninstall" | "none") {
        return Err(format!("Ungültige Aktion: {action}"));
    }
    match state.resolve_backend().await {
        Backend::Live { client_id } => {
            state
                .rpc(
                    "setProductActionRequestWithDependencies",
                    json!([product_id, client_id, action]),
                )
                .await?;
            Ok(())
        }
        _ => {
            let mut products = state.mock_products.lock().await;
            if let Some(p) = products
                .iter_mut()
                .find(|p| p.get("productId").and_then(Value::as_str) == Some(product_id.as_str()))
            {
                p["actionRequest"] = json!(action);
            }
            Ok(())
        }
    }
}

/// `product_ids: None` processes ALL pending action requests (sync).
#[tauri::command]
pub async fn kiosk_process_actions(
    state: tauri::State<'_, KioskState>,
    product_ids: Option<Vec<String>>,
) -> Result<(), String> {
    match state.resolve_backend().await {
        Backend::Live { .. } => {
            state
                .rpc("processActionRequests", json!([product_ids]))
                .await?;
            Ok(())
        }
        _ => {
            // Simulate the action processor: complete pending requests after a delay
            let products = Arc::clone(&state.mock_products);
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                let mut products = products.lock().await;
                for p in products.iter_mut() {
                    let id = p.get("productId").and_then(Value::as_str).unwrap_or("");
                    if let Some(ids) = &product_ids {
                        if !ids.iter().any(|i| i == id) {
                            continue;
                        }
                    }
                    match p.get("actionRequest").and_then(Value::as_str) {
                        Some("setup") => {
                            let version = p["versionStr"].clone();
                            p["installationStatus"] = json!("installed");
                            p["installedVerStr"] = version;
                            p["updatePossible"] = json!(false);
                            p["possibleAction"] = json!("uninstall");
                            p["actionResult"] = json!("successful");
                        }
                        Some("uninstall") => {
                            p["installationStatus"] = json!("not_installed");
                            p["installedVerStr"] = json!("");
                            p["possibleAction"] = json!("setup");
                            p["actionResult"] = json!("successful");
                        }
                        _ => continue,
                    }
                    p["actionRequest"] = json!("none");
                }
            });
            Ok(())
        }
    }
}

/// Product icons are not part of the kiosk JSON-RPC API — the official kiosk
/// ships them as plain files (depot: `opsi-client-kiosk/files/app/ock_custom/
/// product_icons`, copied to the local install dir by the kiosk's setup
/// script). Search order:
///   1. `OPSI_KIOSK_ICON_DIR` env var (if set)
///   2. `%ProgramData%\opsi-kiosk\icons`
///   3. `icons` directory next to the executable
///   4. local opsi-client-kiosk installation (`ock_custom\product_icons`)
///   5. depot share `\\<depot>\opsi_depot\opsi-client-kiosk\files\app\ock_custom\product_icons`
/// File name convention: `<productId>.png|ico|jpg|jpeg|svg|webp`.
#[tauri::command]
pub fn kiosk_get_icons(
    product_ids: Vec<String>,
    depot_id: Option<String>,
) -> std::collections::HashMap<String, String> {
    use base64::Engine;

    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = std::env::var("OPSI_KIOSK_ICON_DIR") {
        dirs.push(dir.into());
    }
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(std::path::Path::new(&program_data).join("opsi-kiosk").join("icons"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("icons"));
        }
    }
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(pf) = std::env::var(var) {
            let pf = std::path::Path::new(&pf);
            dirs.push(pf.join("opsi-client-kiosk").join("ock_custom").join("product_icons"));
            dirs.push(
                pf.join("opsi.org")
                    .join("opsi-client-kiosk")
                    .join("ock_custom")
                    .join("product_icons"),
            );
        }
    }
    if let Some(depot) = depot_id.filter(|d| !d.is_empty() && !d.contains(['/', '\\'])) {
        dirs.push(
            format!(r"\\{depot}\opsi_depot\opsi-client-kiosk\files\app\ock_custom\product_icons")
                .into(),
        );
    }
    // skip dirs that don't exist so unreachable UNC paths only cost one probe
    let dirs: Vec<_> = dirs.into_iter().filter(|d| d.is_dir()).collect();

    let mut result = std::collections::HashMap::new();
    for id in product_ids {
        // product ids are validated host-side, but never build paths from
        // anything containing separators or traversal
        if id.contains(['/', '\\', ':']) || id.contains("..") {
            continue;
        }
        'dirs: for dir in &dirs {
            for (ext, mime) in [
                ("png", "image/png"),
                ("ico", "image/x-icon"),
                ("jpg", "image/jpeg"),
                ("jpeg", "image/jpeg"),
                ("svg", "image/svg+xml"),
                ("webp", "image/webp"),
            ] {
                let path = dir.join(format!("{id}.{ext}"));
                if let Ok(bytes) = std::fs::read(&path) {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                    result.insert(id.clone(), format!("data:{mime};base64,{b64}"));
                    break 'dirs;
                }
            }
        }
    }
    result
}

fn mock_product(
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

fn mock_catalog() -> Vec<Value> {
    vec![
        mock_product("7zip", "7-Zip", "Packprogramm für ZIP-, 7z- und weitere Archivformate.", "24.09", Some("23.01")),
        mock_product("firefox-esr", "Mozilla Firefox ESR", "Webbrowser in der Extended-Support-Variante für Unternehmen.", "128.11.0", Some("128.11.0")),
        mock_product("vlc", "VLC Media Player", "Medienwiedergabe für nahezu alle Audio- und Videoformate.", "3.0.21", None),
        mock_product("keepassxc", "KeePassXC", "Lokaler Passwort-Manager mit KeePass-kompatiblem Datenbankformat.", "2.7.10", None),
        mock_product("greenshot", "Greenshot", "Screenshots erstellen, kommentieren und weitergeben.", "1.3.300", Some("1.3.300")),
        mock_product("notepadplusplus", "Notepad++", "Texteditor mit Syntax-Hervorhebung für viele Sprachen.", "8.8.1", None),
        mock_product("pdf24", "PDF24 Creator", "PDF-Dateien erstellen, zusammenführen und konvertieren.", "11.26.0", None),
    ]
}
