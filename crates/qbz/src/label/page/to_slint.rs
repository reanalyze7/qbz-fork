//! Slint item mapping for the landing page's rows/carousels.

use slint::ModelRc;

use super::{ArtistSlim, LabelSlim, PlaylistSlim, TopTrack};
use crate::album_map::{to_item, AlbumCard};
use crate::{DiscoverSection, SearchPlaylistItem, SlimItem, TrackItem};

pub(super) fn top_track_to_item(t: &TopTrack) -> TrackItem {
    TrackItem {
        // Label landing is Qobuz-only; stamp on the row's artist + album id.
        is_blacklisted: crate::artist_blacklist::stamp_row(
            "qobuz",
            &[t.artist_id.as_str()],
            Some(t.album_id.as_str()),
        ),
        id: t.id.clone().into(),
        number: "".into(),
        title: t.title.clone().into(),
        artist: t.artist.clone().into(),
        album: t.album.clone().into(),
        duration: t.duration.clone().into(),
        quality_tier: t.quality_tier.clone().into(),
        quality_detail: t.quality_detail.clone().into(),
        explicit: false,
        selected: false,
        artwork_url: t.artwork_url.clone().into(),
        artwork: slint::Image::default(),
        is_favorite: crate::fav_cache::is_favorite(&t.id),
        artist_id: t.artist_id.clone().into(),
        album_id: t.album_id.clone().into(),
        removing: false,
        cache_status: if crate::offline_cache::is_cached(&t.id) { 3 } else { 0 },
        cache_progress: 0.0,
        source: "qobuz".into(),
        unlocking: false,
        // Disc grouping is album-detail only; flat lists carry none.
        disc_header_number: 0,
        // Work grouping is album-detail only too.
        work_header: "".into(),
        work_composer_name: "".into(),
        work_composer_id: "".into(),
    }
}

pub(super) fn playlist_to_item(p: &PlaylistSlim) -> SearchPlaylistItem {
    SearchPlaylistItem {
        id: p.id.clone().into(),
        title: p.title.clone().into(),
        subtitle: p.subtitle.clone().into(),
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_playlist_row_pinned when a pin toggles anywhere).
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
        // Label-landing playlist cards carry no category subtag, and a
        // transparent dominant-colour is the sentinel for "no letterbox":
        // the collage keeps the legacy cover-fit (the contain + dominant-
        // colour treatment is Discover-only).
        category: "".into(),
        dominant_color: slint::Color::from_argb_u8(0, 0, 0, 0),
        // Label-landing playlists are foreign Qobuz playlists → follow + copy.
        is_owned: false,
        is_following: false,
        is_copied: false,
    }
}

pub(super) fn artist_to_item(a: &ArtistSlim) -> SlimItem {
    SlimItem {
        id: a.id.clone().into(),
        title: a.name.clone().into(),
        subtitle: "".into(),
        rank: "".into(),
        artwork_url: a.image_url.clone().into(),
        artwork: slint::Image::default(),
        following: false,
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_artist_row_pinned when a pin toggles anywhere).
        is_pinned: crate::pinned::is_pinned("artist", &a.id),
    }
}

pub(super) fn label_to_item(l: &LabelSlim) -> SlimItem {
    SlimItem {
        id: l.id.clone().into(),
        title: l.name.clone().into(),
        subtitle: "".into(),
        rank: "".into(),
        artwork_url: l.image_url.clone().into(),
        artwork: slint::Image::default(),
        following: l.following,
        // Labels are not a pinnable kind (the pinned store admits
        // album/artist/playlist only) — never mark them pinned.
        is_pinned: false,
    }
}

pub(super) fn section(title: &str, cards: &[AlbumCard]) -> DiscoverSection {
    DiscoverSection {
        title: title.into(),
        endpoint: "".into(),
        albums: ModelRc::new(slint::VecModel::from(
            cards.iter().cloned().map(to_item).collect::<Vec<_>>(),
        )),
    }
}
