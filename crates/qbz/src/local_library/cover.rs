//! Best-effort on-disk folder cover resolution — shared by the Folders tree
//! detail pane and the playback queue-track fallback.

/// Best-effort on-disk cover for a folder. The index often has no
/// `artwork_path` (no embedded art + no backfill), yet a cover image sits in
/// the folder under any of a dozen names. Priority: a known cover stem
/// (cover/folder/front/art/album/…), then `<foldername>.<ext>` (a file named
/// after the album), then the first image file as a last resort. Case- and
/// extension-insensitive. Returns an absolute path; must run off the UI thread.
pub fn find_folder_cover(folder: &std::path::Path) -> Option<String> {
    const STEMS: &[&str] = &[
        "cover",
        "folder",
        "front",
        "art",
        "album",
        "albumart",
        "albumartsmall",
        "thumb",
        "artwork",
        "scan",
        "booklet",
        "title",
    ];
    const EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff"];
    let is_img = |p: &std::path::Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| EXTS.iter().any(|x| x.eq_ignore_ascii_case(e)))
            .unwrap_or(false)
    };
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_img(p))
        .collect();
    if entries.is_empty() {
        return None;
    }
    entries.sort();
    let stem_lower = |p: &std::path::Path| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default()
    };
    let folder_name = folder
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let by_stem = entries
        .iter()
        .find(|p| STEMS.contains(&stem_lower(p).as_str()));
    let by_name = entries
        .iter()
        .find(|p| !folder_name.is_empty() && stem_lower(p) == folder_name);
    by_stem
        .or(by_name)
        .cloned()
        .or_else(|| entries.into_iter().next())
        .map(|p| p.to_string_lossy().into_owned())
}
