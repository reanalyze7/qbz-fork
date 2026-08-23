//! Plain `Send` row data + the small formatting helpers shared by the
//! refresh pipeline and the artwork-reuse mapper.

use qbz_models::QueueTrack;

/// Plain `Send` row data built off the UI thread; the non-`Send`
/// `QueueItem` (holds a `slint::Image`) is constructed inside the event
/// loop from this.
pub(super) struct RowData {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) artist: String,
    pub(super) duration: String,
    pub(super) explicit: bool,
    pub(super) artwork_url: String,
    pub(super) playing: bool,
    pub(super) is_ephemeral: bool,
}

/// `M:SS` duration string.
pub(super) fn fmt_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Title with the Qobuz version suffix appended, matching the Tauri
/// `formatTrackTitle` behaviour.
pub(super) fn display_title(track: &QueueTrack) -> String {
    match track.version.as_deref().filter(|v| !v.is_empty()) {
        Some(version) => format!("{} ({version})", track.title),
        None => track.title.clone(),
    }
}

pub(super) fn row_from(track: &QueueTrack, playing: bool) -> RowData {
    RowData {
        id: track.id.to_string(),
        title: display_title(track),
        artist: track.artist.clone(),
        duration: fmt_duration(track.duration_secs),
        explicit: track.parental_warning,
        artwork_url: track.artwork_url.clone().unwrap_or_default(),
        playing,
        is_ephemeral: track.source.as_deref() == Some("ephemeral")
            || crate::ephemeral::is_ephemeral_id(track.id as i64),
    }
}
