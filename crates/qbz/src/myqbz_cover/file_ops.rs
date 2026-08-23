//! Blocking file-system bodies: upload (validate → resize/save → persist →
//! delete-prev) and remove (read prev → clear → delete prev).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use image::imageops::FilterType;

use super::db::{get_prev_path, set_custom_artwork};

/// The four extensions the Tauri picker accepts (spec §7.1 step 1).
const ALLOWED_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// Epoch SECONDS (NOT ms) — the custom-cover filename timestamp is in seconds
/// (spec §1.6 / §7.1 step 4).
fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Keep ascii-alphanumeric / `-` / `_`; everything else becomes `_` (spec
/// §7.1 step 5). The collection id is a UUID, so this is normally a no-op.
fn safe_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Decode `source`, resize to 1000×1000 (Lanczos3), and save as a JPEG at
/// `dest`. Returns an error string on any failure (decode / resize / save).
fn resize_and_save(source: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(source).map_err(|e| format!("decode failed: {e}"))?;
    let resized = img.resize(1000, 1000, FilterType::Lanczos3);
    resized
        .to_rgb8()
        .save_with_format(dest, image::ImageFormat::Jpeg)
        .map_err(|e| format!("save failed: {e}"))
}

/// The blocking upload body (extension validation → resize/save → persist →
/// delete-prev). Returns Ok(dest) or Err(reason). Mirrors the Tauri command
/// step order exactly (persist BEFORE deleting the previous file, and only
/// delete when it differs).
pub(super) fn do_upload(id: &str, source_path: &str) -> Result<String, String> {
    let source = PathBuf::from(source_path);
    if !source.exists() {
        return Err("source file not found".to_string());
    }
    // 1 — extension validation.
    let ext_ok = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false);
    if !ext_ok {
        return Err("unsupported image type".to_string());
    }

    // 2 — read previous path (to delete after persist).
    let prev = get_prev_path(id);

    // 3-6 — build the destination filename in the shared artwork cache dir.
    let artwork_dir = qbz_library::get_artwork_cache_dir();
    let filename = format!("mixtape_custom_{}_{}.jpg", safe_id(id), epoch_secs());
    let dest = artwork_dir.join(&filename);

    // 7 — decode + resize + save.
    resize_and_save(&source, &dest)?;

    // 8 — persist the new path.
    let dest_str = dest.to_string_lossy().to_string();
    if !set_custom_artwork(id, Some(&dest_str)) {
        // Persist failed — clean up the orphan we just wrote, surface the error.
        let _ = std::fs::remove_file(&dest);
        return Err("failed to save cover".to_string());
    }

    // 9 — delete the previous file AFTER persist, only if it differs.
    if let Some(prev) = prev {
        if prev != dest_str {
            let _ = std::fs::remove_file(&prev);
        }
    }

    Ok(dest_str)
}

/// The blocking remove body: read previous → clear → delete prev file.
pub(super) fn do_remove(id: &str) -> Result<(), String> {
    let prev = get_prev_path(id);
    if !set_custom_artwork(id, None) {
        return Err("failed to clear cover".to_string());
    }
    if let Some(prev) = prev {
        let _ = std::fs::remove_file(&prev);
    }
    Ok(())
}
