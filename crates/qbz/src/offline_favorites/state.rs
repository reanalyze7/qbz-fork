//! Shared rail state + row/queue-track construction helpers.

use std::sync::{LazyLock, Mutex};

use qbz_models::QueueTrack;
use qbz_offline_cache::CachedTrackInfo;

/// The rail's queue, in display order — rebuilt by [`super::load::load`],
/// consumed by [`super::play::play`] (clicking a row plays the rail from that
/// row, mirroring the `play_tracks` track-list semantics).
pub(super) static RAIL_QUEUE: LazyLock<Mutex<Vec<QueueTrack>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Worker-built, `Send` row data; the cover is pre-decoded to size on the
/// worker (`DecodedPixels` is `Send`) and the `slint::Image` is built on the
/// UI thread — the offline_manager pattern.
pub(super) struct RowData {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover: Option<crate::artwork::DecodedPixels>,
}

/// Rail rows (SlimCard) render their cover at 44px; decode to the rows tier
/// so the model never holds full-resolution sources.
pub(super) const COVER_DECODE_SIZE: u32 = 96;

/// kHz normalization: Qobuz metadata carries kHz (96.0), library rows Hz
/// (96000.0) — same defensive rule as `local_queue_track`.
pub(super) fn khz(rate: Option<f64>) -> Option<f64> {
    rate.map(|r| if r >= 1000.0 { r / 1000.0 } else { r })
}

/// QueueTrack from an offline-cache index row — mirrors
/// `playback::local_queue_track`'s offline-copy arm: the real Qobuz id,
/// `source = "qobuz_download"`, `is_local = true` (playback then routes
/// through the offline cache tier), `file://` artwork from the offline
/// cover chain.
pub(super) fn index_queue_track(row: &CachedTrackInfo, cover: &str) -> QueueTrack {
    QueueTrack {
        id: row.track_id,
        title: row.title.clone(),
        version: None,
        artist: row.artist.clone(),
        album: row.album.clone().unwrap_or_default(),
        album_version: None,
        duration_secs: row.duration_secs,
        artwork_url: (!cover.is_empty()).then(|| format!("file://{cover}")),
        hires: row.bit_depth.map(|d| d > 16).unwrap_or(false),
        bit_depth: row.bit_depth,
        sample_rate: khz(row.sample_rate),
        is_local: true,
        album_id: row.album_id.clone(),
        artist_id: None,
        streamable: true,
        source: Some("qobuz_download".to_string()),
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}
