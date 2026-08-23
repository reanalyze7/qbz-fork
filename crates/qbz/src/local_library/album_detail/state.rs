//! Cached versions of the currently-open local album + the in-place track
//! filter, plus the small quality/duration formatters used by the load and
//! apply paths.

use std::sync::{LazyLock, Mutex};

/// The open local album's versions (label, tracks) — a "version" is a distinct
/// physical copy (= a distinct source directory). Cached so the version picker
/// switches without a DB round-trip. Splitting by directory is what stops two
/// copies of the same album from merging into a duplicated track list.
static ALBUM_VERSIONS: LazyLock<Mutex<Vec<(String, Vec<qbz_library::LocalTrack>)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub(crate) fn album_versions(
) -> std::sync::MutexGuard<'static, Vec<(String, Vec<qbz_library::LocalTrack>)>> {
    ALBUM_VERSIONS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Client-side track filter for the open local album (mirrors the Qobuz album
/// view's track search). Applied over the current version's tracks at render.
static ALBUM_QUERY: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

pub(crate) fn album_query() -> String {
    ALBUM_QUERY.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub(crate) fn set_album_query(q: String) {
    *ALBUM_QUERY.lock().unwrap_or_else(|e| e.into_inner()) = q;
}

pub(crate) fn fmt_album_duration(secs: u64) -> String {
    let mins = secs / 60;
    if mins >= 60 {
        let h = (mins / 60).to_string();
        let m = (mins % 60).to_string();
        qbz_i18n::t_args("{} h {} min", &[&h, &m])
    } else {
        qbz_i18n::t_args("{} min", &[&mins.to_string()])
    }
}

/// Quality rank for ordering versions (hi-res first).
pub(crate) fn version_rank(t: &qbz_library::LocalTrack) -> (u32, u64) {
    (t.bit_depth.unwrap_or(0), t.sample_rate as u64)
}

/// A version's picker label: "24-bit / 96 kHz · FLAC" (quality + format).
pub(crate) fn version_label(tracks: &[qbz_library::LocalTrack]) -> String {
    match tracks.first() {
        Some(t) => {
            let (detail, _) = crate::local_library::albums::map::local_quality(
                t.bit_depth,
                t.sample_rate,
            );
            let fmt = t.format.to_string();
            if detail.is_empty() {
                fmt
            } else {
                format!("{detail} · {fmt}")
            }
        }
        None => String::new(),
    }
}

/// A version's source ("user" | "qobuz_download" | "") — drives the
/// picker's source icon.
pub(crate) fn version_source(tracks: &[qbz_library::LocalTrack]) -> String {
    tracks
        .first()
        .and_then(|t| t.source.clone())
        .unwrap_or_default()
}
