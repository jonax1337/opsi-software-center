//! JSON-RPC client for the opsiclientd kiosk endpoint (https://localhost:4441/kiosk).
//!
//! The endpoint allows unauthenticated access from localhost when
//! `control_server.kiosk_api_active` is enabled (opsi default). It exposes the
//! same interface the official opsi-client-kiosk uses; the relevant methods are
//! `getClientId`, `getKioskProductInfosForClient`,
//! `setProductActionRequestWithDependencies` and `processActionRequests`.
//!
//! The in-memory demo catalog (see `mock.rs`) is used only when forced with
//! `OPSI_KIOSK_MOCK=1`, or in debug builds when opsiclientd is unreachable.
//! Release builds never show fake products: an unreachable opsiclientd is
//! reported as "not connected" and retried on the next call.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::mock::MockCatalog;

const KIOSK_URL: &str = "https://localhost:4441/kiosk";
const RPC_TIMEOUT: Duration = Duration::from_secs(15);
const MOCK_ENV_VAR: &str = "OPSI_KIOSK_MOCK";
/// Developer machines usually have no opsiclientd — fall back to the demo
/// catalog there, but never in a release build.
const FALL_BACK_TO_MOCK: bool = cfg!(debug_assertions);

#[derive(Clone)]
enum Backend {
    Live { client_id: String },
    Mock,
}

/// The action requests the UI may set. Anything else is rejected while
/// deserializing the command arguments.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionRequest {
    Setup,
    Uninstall,
    None,
}

impl ActionRequest {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Uninstall => "uninstall",
            Self::None => "none",
        }
    }
}

pub struct KioskState {
    http: reqwest::Client,
    rpc_id: AtomicU64,
    /// Resolved once successfully, then cached. Failures are not cached so a
    /// late-starting opsiclientd (autostart at login) is picked up on retry.
    backend: Mutex<Option<Backend>>,
    mock: MockCatalog,
}

impl KioskState {
    pub fn new() -> Self {
        Self {
            // opsiclientd serves a self-signed certificate on localhost
            http: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(RPC_TIMEOUT)
                .build()
                .expect("failed to build http client"),
            rpc_id: AtomicU64::new(1),
            backend: Mutex::new(None),
            mock: MockCatalog::new(),
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
            return Err(format!("opsiclientd antwortete bei {method} mit HTTP {status}"));
        }
        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| format!("Ungültige JSON-RPC-Antwort auf {method}: {e}"))?;
        if let Some(err) = parsed.get("error").filter(|e| !e.is_null()) {
            let msg = err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unbekannter Fehler");
            return Err(format!("opsi-Fehler bei {method}: {msg}"));
        }
        Ok(parsed.get("result").cloned().unwrap_or(Value::Null))
    }

    async fn backend(&self) -> Result<Backend, String> {
        let mut cached = self.backend.lock().await;
        if let Some(backend) = cached.as_ref() {
            return Ok(backend.clone());
        }
        let resolved = self.resolve_backend().await?;
        *cached = Some(resolved.clone());
        Ok(resolved)
    }

    async fn resolve_backend(&self) -> Result<Backend, String> {
        if std::env::var(MOCK_ENV_VAR).as_deref() == Ok("1") {
            return Ok(Backend::Mock);
        }
        match self.rpc("getClientId", json!([])).await {
            Ok(Value::String(client_id)) if !client_id.is_empty() => {
                Ok(Backend::Live { client_id })
            }
            Ok(other) => Err(format!("opsiclientd lieferte keine Client-ID: {other}")),
            Err(_) if FALL_BACK_TO_MOCK => Ok(Backend::Mock),
            Err(e) => Err(e),
        }
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
    /// Why the connection failed; empty when connected.
    error: String,
}

impl KioskStatus {
    fn disconnected(error: String) -> Self {
        Self {
            connected: false,
            mock: false,
            client_id: String::new(),
            depot_id: String::new(),
            version: String::new(),
            error,
        }
    }

    fn mock() -> Self {
        Self {
            connected: false,
            mock: true,
            client_id: "demo-client.example.org".into(),
            depot_id: "opsi-depot.example.org".into(),
            version: "mock".into(),
            error: String::new(),
        }
    }
}

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// Never fails: an unreachable opsiclientd is a status, not an error, so the
/// UI can show it and offer a retry.
#[tauri::command]
pub async fn kiosk_status(state: tauri::State<'_, KioskState>) -> Result<KioskStatus, String> {
    let client_id = match state.backend().await {
        Ok(Backend::Live { client_id }) => client_id,
        Ok(Backend::Mock) => return Ok(KioskStatus::mock()),
        Err(e) => return Ok(KioskStatus::disconnected(e)),
    };
    // depot and agent version are informational — show the client even when
    // this call fails
    let config = state
        .rpc("getConfigDataFromOpsiclientd", json!([true, false]))
        .await
        .unwrap_or(Value::Null);
    Ok(KioskStatus {
        connected: true,
        mock: false,
        client_id,
        depot_id: str_field(&config, "depot_id"),
        version: str_field(&config, "opsiclientd_version"),
        error: String::new(),
    })
}

#[tauri::command]
pub async fn kiosk_get_products(state: tauri::State<'_, KioskState>) -> Result<Vec<Value>, String> {
    match state.backend().await? {
        Backend::Live { client_id } => {
            match state
                .rpc("getKioskProductInfosForClient", json!([client_id, false]))
                .await?
            {
                Value::Array(products) => Ok(products),
                other => Err(format!("Unerwartetes Antwortformat: {other}")),
            }
        }
        Backend::Mock => Ok(state.mock.products().await),
    }
}

#[tauri::command]
pub async fn kiosk_set_action(
    state: tauri::State<'_, KioskState>,
    product_id: String,
    action: ActionRequest,
) -> Result<(), String> {
    match state.backend().await? {
        Backend::Live { client_id } => {
            state
                .rpc(
                    "setProductActionRequestWithDependencies",
                    json!([product_id, client_id, action.as_str()]),
                )
                .await?;
        }
        Backend::Mock => state.mock.set_action(&product_id, action).await,
    }
    Ok(())
}

/// `product_ids: None` processes ALL pending action requests (sync).
#[tauri::command]
pub async fn kiosk_process_actions(
    state: tauri::State<'_, KioskState>,
    product_ids: Option<Vec<String>>,
) -> Result<(), String> {
    match state.backend().await? {
        Backend::Live { .. } => {
            state
                .rpc("processActionRequests", json!([product_ids]))
                .await?;
        }
        Backend::Mock => state.mock.process(product_ids),
    }
    Ok(())
}
