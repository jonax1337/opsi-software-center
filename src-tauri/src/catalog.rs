//! Optional catalog metadata the kiosk API does not provide: publisher,
//! category and "featured" flag per product — what Company Portal shows on
//! tiles, in the sort/category pickers and in the home-page banner.
//!
//! Read from `catalog.json` in `%ProgramData%\opsi-kiosk` (written by the opsi
//! package; only admins can write there), then next to the executable:
//!
//! ```json
//! { "7zip": { "publisher": "Igor Pavlov", "category": "Werkzeuge", "featured": true } }
//! ```
//!
//! Every field is optional. Products without an entry simply show no publisher.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const CATALOG_FILE: &str = "catalog.json";

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CatalogEntry {
    pub publisher: String,
    pub category: String,
    pub featured: bool,
}

pub type Catalog = HashMap<String, CatalogEntry>;

/// productId → catalog entry. Missing file = empty catalog; a malformed file
/// is reported so the admin notices the typo. Demo mode falls back to the
/// built-in demo catalog when no file is deployed.
#[tauri::command]
pub async fn kiosk_get_catalog(
    state: tauri::State<'_, crate::kiosk::KioskState>,
) -> Result<Catalog, String> {
    let catalog = crate::winproc::blocking(|| load_catalog(&candidate_files())).await??;
    match catalog {
        Some(catalog) => Ok(catalog),
        None if state.is_mock().await => Ok(crate::mock::demo_catalog()),
        None => Ok(Catalog::new()),
    }
}

fn candidate_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(data) = crate::config::data_dir() {
        files.push(data.join(CATALOG_FILE));
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        files.push(exe_dir.join(CATALOG_FILE));
    }
    files
}

/// The first existing file wins; `None` when there is none.
fn load_catalog(files: &[PathBuf]) -> Result<Option<Catalog>, String> {
    let Some(file) = files.iter().find(|f| f.is_file()) else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| format!("{} ist kein gültiger Katalog: {e}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("opsi-kiosk-catalog-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_is_no_catalog() {
        let dir = temp_dir("missing");
        let result = load_catalog(&[dir.join(CATALOG_FILE)]);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn reads_partial_entries_with_defaults() {
        let dir = temp_dir("partial");
        let file = dir.join(CATALOG_FILE);
        std::fs::write(&file, r#"{ "vlc": { "publisher": "VideoLAN" } }"#).unwrap();
        let catalog = load_catalog(&[file]).unwrap().unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        let vlc = &catalog["vlc"];
        assert_eq!(vlc.publisher, "VideoLAN");
        assert_eq!(vlc.category, "");
        assert!(!vlc.featured);
    }

    #[test]
    fn malformed_file_is_an_error_naming_the_file() {
        let dir = temp_dir("malformed");
        let file = dir.join(CATALOG_FILE);
        std::fs::write(&file, "{ not json").unwrap();
        let err = load_catalog(&[file]).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(err.contains(CATALOG_FILE), "{err}");
    }
}
