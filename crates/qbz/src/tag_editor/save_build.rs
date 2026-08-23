//! Validation for `save_tags` (UI-thread half). Payload-struct building
//! lives in `save_payload.rs`.

use slint::Weak;

use crate::{AppWindow, TagTrackEdit};

use super::parse_year;

/// Validate the editor's current fields. Returns the parsed year and the
/// trimmed album directory on success; toasts + returns `None` on failure.
pub(super) fn validate(
    weak: &Weak<AppWindow>,
    group_key: &str,
    album_title: &str,
    year_input: &str,
    direct: bool,
    rows: &[TagTrackEdit],
) -> Option<(Option<u32>, String)> {
    if album_title.is_empty() {
        crate::toast::error_weak(weak, qbz_i18n::t("Album title is required"));
        return None;
    }
    if rows.iter().any(|r| r.title.trim().is_empty()) {
        crate::toast::error_weak(weak, qbz_i18n::t("Every track needs a title"));
        return None;
    }
    let year = match parse_year(year_input) {
        Ok(y) => y,
        Err(()) => {
            crate::toast::error_weak(weak, qbz_i18n::t("Year must be a number"));
            return None;
        }
    };
    let album_dir = group_key.trim().to_string();
    if !std::path::Path::new(&album_dir).is_dir() {
        crate::toast::error_weak(weak, qbz_i18n::t("Album folder not found on disk"));
        return None;
    }
    if direct && rows.iter().any(|r| r.has_cue) {
        crate::toast::error_weak(
            weak,
            qbz_i18n::t("Writing tags to files isn't supported for CUE albums; use sidecar mode"),
        );
        return None;
    }
    Some((year, album_dir))
}
