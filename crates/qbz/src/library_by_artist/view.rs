//! Slint `ModelRc` builders for the artist-library toggle's track/album lists.

use slint::{ModelRc, VecModel};

use crate::album_map::{self, AlbumCard};
use crate::favorites::TrackCard;
use crate::{AlbumCardItem, TrackItem};

/// Convert stored album cards to the Slint model (reuses `album_map::to_item`,
/// which stamps favorite/pin state).
pub fn album_items(albums: &[AlbumCard]) -> ModelRc<AlbumCardItem> {
    let rows: Vec<AlbumCardItem> = albums.iter().cloned().map(album_map::to_item).collect();
    ModelRc::new(VecModel::from(rows))
}

/// Convert stored track cards to the Slint model (mirrors the favorites
/// apply_favorites mapping; everything here is, by definition, a favorite).
pub fn track_items(tracks: &[TrackCard]) -> ModelRc<TrackItem> {
    let rows: Vec<TrackItem> = tracks
        .iter()
        .cloned()
        .map(|t| TrackItem {
            is_blacklisted: false,
            id: t.id.clone().into(),
            number: "".into(),
            title: t.title.into(),
            artist: t.artist.into(),
            album: t.album.into(),
            duration: t.duration.into(),
            quality_tier: t.quality_tier.into(),
            quality_detail: t.quality_detail.into(),
            explicit: t.explicit,
            selected: false,
            artwork_url: t.artwork_url.into(),
            artwork: slint::Image::default(),
            is_favorite: true,
            artist_id: t.artist_id.into(),
            album_id: t.album_id.into(),
            removing: false,
            cache_status: 0,
            cache_progress: 0.0,
            source: "qobuz".into(),
            unlocking: false,
            disc_header_number: 0,
            work_header: "".into(),
            work_composer_name: "".into(),
            work_composer_id: "".into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}
