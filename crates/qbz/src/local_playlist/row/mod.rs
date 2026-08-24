//! The E11 shared row-identity contract: resolved row types, and the
//! duration/queue/item builders used by both the LOCAL detail and the
//! offline MIXED detail.

mod build;
mod item;
mod queue;

pub(crate) use build::build_row_models;
pub(crate) use queue::row_queue_track;

/// One resolved, renderable row (Send — built on the worker).
pub enum RowItem {
    /// Full catalog track (online fetch).
    Qobuz(Box<qbz_models::Track>),
    /// Offline-cache metadata (D11: a Qobuz row renders offline ONLY when
    /// this metadata source exists; un-cached rows are filtered out).
    Cached {
        track_id: u64,
        title: String,
        artist: String,
        album: String,
        duration_secs: u64,
        bit_depth: Option<u32>,
        sample_rate: Option<f64>,
        /// On-disk cover thumb (B5: index `artwork_path` / CMAF `cover.jpg`),
        /// loaded through the local-file artwork path like Local rows.
        artwork_path: Option<String>,
    },
    /// Local file resolved from library.db by path.
    Local(Box<qbz_library::LocalTrack>),
    /// Local file row whose metadata resolve failed but whose file EXISTS
    /// on disk — renders with a filename fallback (D11 hiding is for
    /// unavailable-offline QOBUZ rows, not for a file that is right there).
    /// Not playable until the row is back in the library index.
    LocalFile { path: String },
    /// A ref that cannot resolve right now: a `qobuz_track_id` outside the
    /// catalog id range (the legacy untyped-drag bug stored synthetic
    /// 2^40-namespaced row ids as Qobuz ids). Renders an HONEST, selectable
    /// (removable) row instead of hiding — D11 hiding is for genuinely-
    /// offline Qobuz rows, not for refs that can never heal on their own.
    Unresolved {
        /// "qobuz" (out-of-range id — permanent garbage).
        kind: &'static str,
        /// The raw stored ref, shown so the user knows WHAT is broken.
        reference: String,
    },
}

pub struct LoadedRow {
    pub position: i32,
    pub item: RowItem,
}

pub struct LocalPlaylistData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub offline_only: bool,
    pub custom_artwork_path: Option<String>,
    pub rows: Vec<LoadedRow>,
}

pub(crate) fn mmss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(crate) fn total_duration_label(rows: &[LoadedRow]) -> String {
    let secs: u64 = rows
        .iter()
        .map(|r| match &r.item {
            RowItem::Qobuz(t) => t.duration as u64,
            RowItem::Cached { duration_secs, .. } => *duration_secs,
            RowItem::Local(t) => t.duration_secs,
            RowItem::LocalFile { .. } => 0,
            RowItem::Unresolved { .. } => 0,
        })
        .sum();
    let mins = secs / 60;
    if mins >= 60 {
        let h = (mins / 60).to_string();
        let m = (mins % 60).to_string();
        qbz_i18n::t_args("{} h {} min", &[&h, &m])
    } else {
        qbz_i18n::t_args("{} min", &[&mins.to_string()])
    }
}
