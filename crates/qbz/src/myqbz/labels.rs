//! Pure string/formatting helpers — no Slint types.

use qbz_models::mixtape::CollectionKind;

pub(super) fn kind_str(kind: CollectionKind) -> &'static str {
    match kind {
        CollectionKind::Mixtape => "mixtape",
        CollectionKind::Collection => "collection",
        CollectionKind::ArtistCollection => "artist_collection",
    }
}

/// Eyebrow label, uppercase (Tauri `labelFor` / `mixtapes.label`).
pub(super) fn label_for(kind: CollectionKind) -> String {
    match kind {
        CollectionKind::Mixtape => qbz_i18n::t("MIXTAPE"),
        CollectionKind::Collection => qbz_i18n::t("COLLECTION"),
        CollectionKind::ArtistCollection => qbz_i18n::t("ARTIST"),
    }
}

/// `mixtapes.albumCount` ICU plural — "1 album" / "N albums". Always "album(s)"
/// regardless of item_type (1:1 with the PSD).
pub(super) fn album_count_label(count: usize) -> String {
    qbz_i18n::tf("{} album", "{} albums", count as i64, &[&count.to_string()])
}

/// Pre-downscale a Qobuz cover URL to a per-cell target size, mirroring the
/// mosaic's `smallQobuzUrl` (regex-swap `_<old>.jpg` → `_<target>.jpg`). Used
/// before handing URLs to the image loader so we never pull 600px covers for
/// ~60-92px cells. Non-Qobuz URLs (local) pass through unchanged.
pub fn small_qobuz_url(url: &str, target: u32) -> String {
    if url.is_empty() {
        return String::new();
    }
    // Lowercase scan for the size token; rewrite in place keeping original case
    // of the rest. Old tokens: 50|100|150|230|300|600|max|org.
    const TOKENS: [&str; 8] = ["_50.jpg", "_100.jpg", "_150.jpg", "_230.jpg", "_300.jpg", "_600.jpg", "_max.jpg", "_org.jpg"];
    let lower = url.to_lowercase();
    for tok in TOKENS {
        if let Some(pos) = lower.rfind(tok) {
            let mut out = String::with_capacity(url.len());
            out.push_str(&url[..pos]);
            out.push_str(&format!("_{target}.jpg"));
            out.push_str(&url[pos + tok.len()..]);
            return out;
        }
    }
    url.to_string()
}

/// Per-cell target size given the mosaic `size` and column count
/// (`cellPx = round(size/cols)`; `<=80 → 50`, `<=200 → 150`, else 300). The
/// grid card mosaic is 184px (2x2 → cell 92 → 150; 3x3 → cell ~61 → 50).
pub(super) fn cell_target(size: u32, cols: u32) -> u32 {
    let cell_px = ((size as f32) / (cols as f32)).round() as u32;
    if cell_px <= 80 {
        50
    } else if cell_px <= 200 {
        150
    } else {
        300
    }
}
