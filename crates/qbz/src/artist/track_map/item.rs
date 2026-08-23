use crate::album::TrackData;
use crate::artist::data::PlaylistSlim;
use crate::{SearchPlaylistItem, TrackItem};

/// Build a Slint `TrackItem` from a controller `TrackData`, stamping
/// favorite/cache status. Shared by Popular Tracks and Appears On (both
/// flat cross-album lists — no disc headers, no per-row blacklist greyout).
pub(crate) fn track_data_to_item(track: TrackData) -> TrackItem {
    TrackItem {
        is_blacklisted: false,
        id: track.id.clone().into(),
        number: track.number.into(),
        title: track.title.into(),
        artist: track.artist.into(),
        album: track.album.clone().into(),
        duration: track.duration.into(),
        quality_tier: track.quality_tier.into(),
        quality_detail: track.quality_detail.into(),
        explicit: track.explicit,
        selected: false,
        artwork_url: track.artwork_url.clone().into(),
        artwork: slint::Image::default(),
        is_favorite: crate::fav_cache::is_favorite(&track.id),
        artist_id: track.artist_id.into(),
        album_id: track.album_id.into(),
        removing: false,
        cache_status: if crate::offline_cache::is_cached(&track.id) { 3 } else { 0 },
        cache_progress: 0.0,
        source: "qobuz".into(),
        unlocking: false,
        disc_header_number: 0,
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}

/// Map an artist-page curated playlist to the shared collage card model
/// (single cover, slot 0 — filled by ArtworkTarget::ArtistPlaylistCover).
/// Mirrors `label::playlist_to_item`.
pub(crate) fn playlist_to_item(p: &PlaylistSlim) -> SearchPlaylistItem {
    SearchPlaylistItem {
        id: p.id.clone().into(),
        title: p.title.clone().into(),
        subtitle: p.subtitle.clone().into(),
        is_pinned: crate::pinned::is_pinned("playlist", &p.id),
        cover_count: if p.image_url.is_empty() { 0 } else { 1 },
        url1: p.image_url.clone().into(),
        url2: "".into(),
        url3: "".into(),
        url4: "".into(),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
        category: "".into(),
        dominant_color: slint::Color::from_argb_u8(0, 0, 0, 0),
        // Artist-page playlists are foreign Qobuz playlists → follow + copy.
        is_owned: false,
        is_following: false,
        is_copied: false,
    }
}
