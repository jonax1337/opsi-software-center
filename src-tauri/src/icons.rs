//! Product icons. They are not part of the kiosk JSON-RPC API — the official
//! kiosk ships them as plain files (depot: `opsi-client-kiosk/files/app/
//! ock_custom/product_icons`). Search order:
//!   1. `OPSI_KIOSK_ICON_DIR` env var (if set)
//!   2. `%ProgramData%\opsi-kiosk\icons` (filled by the opsi package)
//!   3. `icons` directory next to the executable
//!   4. local opsi-client-kiosk installation (`ock_custom\product_icons`)
//!   5. depot share `\\<depot>\opsi_depot\opsi-client-kiosk\files\app\ock_custom\product_icons`
//!
//! File name convention: `<productId>.png|ico|jpg|jpeg|svg|webp`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use base64::Engine;

const ICON_DIR_ENV_VAR: &str = "OPSI_KIOSK_ICON_DIR";

/// Extension → MIME type, in lookup priority order.
const ICON_FORMATS: [(&str, &str); 6] = [
    ("png", "image/png"),
    ("ico", "image/x-icon"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("svg", "image/svg+xml"),
    ("webp", "image/webp"),
];

/// productId → data-URL for every product that has an icon file. Runs off the
/// main thread: probing an unreachable depot share can take seconds. In demo
/// mode, products without an icon file get a generated monogram.
#[tauri::command]
pub async fn kiosk_get_icons(
    state: tauri::State<'_, crate::kiosk::KioskState>,
    product_ids: Vec<String>,
    depot_id: Option<String>,
) -> Result<HashMap<String, String>, String> {
    let ids = product_ids.clone();
    let mut icons = crate::winproc::blocking(move || load_icons(ids, depot_id.as_deref())).await?;
    if state.is_mock().await {
        for id in product_ids {
            if let Some(icon) = crate::mock::demo_icon(&id) {
                icons.entry(id).or_insert(icon);
            }
        }
    }
    Ok(icons)
}

fn load_icons(product_ids: Vec<String>, depot_id: Option<&str>) -> HashMap<String, String> {
    // skip dirs that don't exist so an unreachable UNC path costs one probe
    let dirs: Vec<PathBuf> = candidate_dirs(depot_id)
        .into_iter()
        .filter(|d| d.is_dir())
        .collect();
    product_ids
        .into_iter()
        .filter(|id| is_safe_file_stem(id))
        .filter_map(|id| find_icon(&dirs, &id).map(|url| (id, url)))
        .collect()
}

fn candidate_dirs(depot_id: Option<&str>) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(dir) = std::env::var(ICON_DIR_ENV_VAR) {
        dirs.push(dir.into());
    }
    if let Some(data) = crate::config::data_dir() {
        dirs.push(data.join("icons"));
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        dirs.push(exe_dir.join("icons"));
    }
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(program_files) = std::env::var(var) {
            let program_files = Path::new(&program_files);
            for kiosk_dir in [program_files.join("opsi-client-kiosk"), program_files.join(r"opsi.org\opsi-client-kiosk")] {
                dirs.push(kiosk_dir.join(r"ock_custom\product_icons"));
            }
        }
    }
    if let Some(depot) = depot_id.filter(|d| is_safe_host_name(d)) {
        dirs.push(format!(r"\\{depot}\opsi_depot\opsi-client-kiosk\files\app\ock_custom\product_icons").into());
    }
    dirs
}

/// First icon file for `product_id` across `dirs`, as a data-URL.
fn find_icon(dirs: &[PathBuf], product_id: &str) -> Option<String> {
    dirs.iter().find_map(|dir| {
        ICON_FORMATS.iter().find_map(|(ext, mime)| {
            let bytes = std::fs::read(dir.join(format!("{product_id}.{ext}"))).ok()?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
            Some(format!("data:{mime};base64,{b64}"))
        })
    })
}

/// Product ids are validated by opsi, but never build a path from anything
/// containing separators, drive letters or traversal.
fn is_safe_file_stem(id: &str) -> bool {
    !id.is_empty() && !id.contains(['/', '\\', ':']) && !id.contains("..")
}

fn is_safe_host_name(host: &str) -> bool {
    !host.is_empty() && !host.contains(['/', '\\'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ids_that_could_escape_the_icon_dir() {
        for id in ["", "../secret", "a/b", r"a\b", "C:x", "a..b"] {
            assert!(!is_safe_file_stem(id), "{id:?} must be rejected");
        }
        assert!(is_safe_file_stem("firefox-esr"));
    }

    #[test]
    fn rejects_depot_names_with_separators() {
        assert!(!is_safe_host_name(r"evil\share"));
        assert!(!is_safe_host_name(""));
        assert!(is_safe_host_name("depot.example.org"));
    }

    #[test]
    fn finds_icons_in_priority_order() {
        let root = std::env::temp_dir().join(format!("opsi-kiosk-icons-{}", std::process::id()));
        let (first, second) = (root.join("first"), root.join("second"));
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        std::fs::write(first.join("vlc.svg"), b"<svg/>").unwrap();
        std::fs::write(second.join("vlc.png"), b"png").unwrap();

        let dirs = [first, second];
        let url = find_icon(&dirs, "vlc").unwrap();
        let missing = find_icon(&dirs, "firefox");
        std::fs::remove_dir_all(&root).unwrap();

        assert!(url.starts_with("data:image/svg+xml;base64,"), "earlier dir wins: {url}");
        assert!(missing.is_none());
    }
}
