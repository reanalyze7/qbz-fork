use crate::TrackItem;

pub(super) fn local_file_item(path: &str) -> TrackItem {
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    TrackItem {
        // Bare local file — never blacklisted.
        is_blacklisted: false,
        id: format!("file:{path}").into(),
        number: "".into(),
        title: name.into(),
        artist: "".into(),
        album: "".into(),
        duration: "".into(),
        quality_tier: "".into(),
        quality_detail: "".into(),
        explicit: false,
        selected: false,
        artwork_url: "".into(),
        artwork: slint::Image::default(),
        is_favorite: false,
        artist_id: "".into(),
        album_id: "".into(),
        removing: false,
        cache_status: 0,
        cache_progress: 0.0,
        source: "local".into(),
        unlocking: false,
        // Disc grouping is album-detail only; playlist rows carry none.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}

/// Honest unavailable row: distinct title + the raw stored ref in the album
/// column, selectable so multi-select removal can clear it. Mis-typed qobuz
/// refs get an unparseable id so no drag/pick path can ever re-type them as
/// a catalog id.
pub(super) fn unresolved_item(kind: &str, reference: &str) -> TrackItem {
    TrackItem {
        // Unresolved/unavailable row — never blacklisted.
        is_blacklisted: false,
        id: format!("broken:{kind}:{reference}").into(),
        number: "".into(),
        title: qbz_i18n::t("Unavailable track").into(),
        artist: qbz_i18n::t("Unknown source").into(),
        album: format!("ref {reference}").into(),
        duration: "".into(),
        quality_tier: "".into(),
        quality_detail: "".into(),
        explicit: false,
        selected: false,
        artwork_url: "".into(),
        artwork: slint::Image::default(),
        is_favorite: false,
        artist_id: "".into(),
        album_id: "".into(),
        removing: false,
        cache_status: 0,
        cache_progress: 0.0,
        source: "".into(),
        unlocking: false,
        // Disc grouping is album-detail only; playlist rows carry none.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}
