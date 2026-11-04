//! "Temporäre Dateien bereinigen": empties the signed-in user's temp folder.
//! Only files created and modified more than a day ago are removed, so
//! installers or apps running right now keep their working files (extractors
//! often keep the archive's old modification time, hence also the creation
//! time); files in use fail to delete and are skipped. Links (symlinks,
//! junctions) are never followed or removed.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Serialize;

use crate::winproc::blocking;

/// Files created or modified more recently than this stay.
const MIN_AGE: Duration = Duration::from_secs(24 * 60 * 60);
/// `Temp\Low` is writable by low-integrity processes (sandboxed browsers);
/// one could swap a checked folder for a junction there, so it stays untouched.
const LOW_INTEGRITY_DIR: &str = "Low";

#[derive(Serialize, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub files: u64,
    pub bytes: u64,
}

#[tauri::command]
pub async fn device_clean_temp() -> Result<CleanupResult, String> {
    blocking(|| {
        let cutoff = SystemTime::now() - MIN_AGE;
        let mut result = CleanupResult::default();
        clean_dir(&temp_root()?, cutoff, &mut result);
        Ok(result)
    })
    .await?
}

/// `%LOCALAPPDATA%\Temp` from the known folder, deliberately not `%TMP%`: the
/// user (or a legacy GPO) can point that at a profile, a share or a drive root,
/// and the cleanup must never sweep those.
#[cfg(windows)]
fn temp_root() -> Result<PathBuf, String> {
    use windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData;
    Ok(crate::winproc::known_folder(&FOLDERID_LocalAppData)?.join("Temp"))
}

#[cfg(not(windows))]
fn temp_root() -> Result<PathBuf, String> {
    Ok(std::env::temp_dir())
}

/// Older than `cutoff` by both creation and modification time.
fn is_older(meta: &fs::Metadata, cutoff: SystemTime) -> bool {
    let before = |time: std::io::Result<SystemTime>| time.is_ok_and(|t| t < cutoff);
    before(meta.created()) && before(meta.modified())
}

/// Deletes every file below `dir` older than `cutoff`, then the old
/// subdirectories that became empty. `dir` itself stays. Entries that cannot
/// be read or deleted (in use, no permission) are skipped on purpose — a
/// partial cleanup is the expected outcome, not an error.
fn clean_dir(dir: &Path, cutoff: SystemTime, result: &mut CleanupResult) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if entry.file_name() == LOW_INTEGRITY_DIR {
            continue;
        }
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else { continue };
        if meta.file_type().is_symlink() {
            continue;
        }
        let is_old = is_older(&meta, cutoff);
        if meta.is_dir() {
            clean_dir(&path, cutoff, result);
            // an app may have just created an empty working folder; a removal
            // also fails while the folder still holds young or locked files
            if is_old {
                let _ = fs::remove_dir(&path);
            }
        } else if is_old && fs::remove_file(&path).is_ok() {
            result.files += 1;
            result.bytes += meta.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("opsi-kiosk-cleanup-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub/deeper")).unwrap();
        fs::write(dir.join("a.tmp"), b"12345").unwrap();
        fs::write(dir.join("sub/deeper/b.tmp"), b"123").unwrap();
        dir
    }

    #[test]
    fn removes_old_files_and_emptied_folders_but_keeps_the_root() {
        let dir = scratch_dir("old");
        let mut result = CleanupResult::default();
        clean_dir(&dir, SystemTime::now() + Duration::from_secs(60), &mut result);

        assert_eq!(result, CleanupResult { files: 2, bytes: 8 });
        assert!(dir.is_dir());
        assert!(!dir.join("sub").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn leaves_the_low_integrity_folder_alone() {
        let dir = scratch_dir("low");
        fs::create_dir_all(dir.join(LOW_INTEGRITY_DIR)).unwrap();
        fs::write(dir.join(LOW_INTEGRITY_DIR).join("c.tmp"), b"1").unwrap();
        let mut result = CleanupResult::default();
        clean_dir(&dir, SystemTime::now() + Duration::from_secs(60), &mut result);

        assert!(dir.join(LOW_INTEGRITY_DIR).join("c.tmp").is_file());
        assert_eq!(result.files, 2);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn keeps_recent_files_and_their_folders() {
        let dir = scratch_dir("recent");
        let mut result = CleanupResult::default();
        clean_dir(&dir, SystemTime::now() - MIN_AGE, &mut result);

        assert_eq!(result, CleanupResult::default());
        assert!(dir.join("sub/deeper/b.tmp").is_file());
        fs::remove_dir_all(&dir).unwrap();
    }
}
