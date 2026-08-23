//! Booklet-goody selection: prefer the PDF format id, else the first goody
//! whose URL ends in `.pdf`.

use qbz_models::Album;

/// Pick the booklet goody: prefer the PDF format id (21), else the first
/// goody whose url/original_url ends in ".pdf". `original_url` (full-size)
/// wins over the thumbnail `url`. The caller gates `has_booklet` on a usable
/// URL — not merely the presence of a goody.
pub(super) fn booklet_url(album: &Album) -> String {
    album
        .goodies
        .as_deref()
        .and_then(|goodies| {
            goodies
                .iter()
                .find(|g| g.file_format_id == Some(21))
                .or_else(|| {
                    goodies.iter().find(|g| {
                        let ends_pdf = |s: &str| s.to_lowercase().ends_with(".pdf");
                        ends_pdf(&g.original_url) || ends_pdf(&g.url)
                    })
                })
        })
        .map(|g| {
            if !g.original_url.is_empty() {
                g.original_url.clone()
            } else {
                g.url.clone()
            }
        })
        .unwrap_or_default()
}
