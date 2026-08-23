//! The [`MetaFields`] struct definition, split out of `fields.rs` (the
//! builder) to keep both files under the line budget.

/// Every value derived from the current track that the meta push, the MPRIS/
/// tray sync, and the artwork loaders need.
pub(super) struct MetaFields {
    pub(super) title: String,
    pub(super) artist: String,
    pub(super) album: String,
    pub(super) album_display: String,
    pub(super) album_id: String,
    pub(super) artist_id: String,
    pub(super) context_kind: String,
    pub(super) context_id: String,
    pub(super) track_id_num: u64,
    pub(super) track_id: String,
    pub(super) is_ephemeral: bool,
    pub(super) source: String,
    pub(super) local_track_id: String,
    pub(super) duration: u64,
    pub(super) bar_artwork: qbz_models::ArtworkRef,
    pub(super) preview_artwork: qbz_models::ArtworkRef,
    pub(super) quality_tier: &'static str,
    pub(super) quality_detail: String,
    pub(super) bit_depth: Option<u32>,
    pub(super) sample_rate: Option<f64>,
    pub(super) album_favorite: bool,
}
