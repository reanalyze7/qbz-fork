//! Pure per-item converters: plain home data -> Slint item structs.

use crate::{AlbumCardItem, SearchPlaylistItem, SlimItem};

use super::super::{CardData, PlaylistCardData, SlimData};

/// Convert one `SlimData` into the Slint `SlimItem` (shared by `apply_home`
/// and `apply_recent_rails`).
pub(crate) fn slim_to_item(slim: SlimData) -> SlimItem {
    SlimItem {
        id: slim.id.into(),
        title: slim.title.into(),
        subtitle: slim.subtitle.into(),
        rank: slim.rank.into(),
        artwork_url: slim.artwork_url.into(),
        artwork: slint::Image::default(),
        following: false,
        // Slim rails (popular / recently played) render pin-less slim rows,
        // not the grid card — nothing here is pinnable.
        is_pinned: false,
    }
}

/// Convert one `CardData` into the Slint `AlbumCardItem`.
pub(crate) fn card_to_item(card: CardData) -> AlbumCardItem {
    AlbumCardItem {
        plays: 0,
        // Favorite heart state from the login-seeded cache (kept live by
        // main::set_album_row_favorite when a favorite toggles anywhere).
        is_favorite: crate::fav_cache::is_album_favorite(&card.id),
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_album_row_pinned when a pin toggles anywhere).
        is_pinned: crate::pinned::is_pinned("album", &card.id),
        id: card.id.into(),
        title: card.title.into(),
        artist: card.artist.into(),
        artist_id: card.artist_id.into(),
        genre: card.genre.into(),
        year: card.year.into(),
        quality_tier: card.quality_tier.into(),
        quality_label: card.quality_label.into(),
        ribbon: card.ribbon.into(),
        ribbon_kind: card.ribbon_kind.into(),
        artwork_url: card.artwork_url.into(),
        artwork: slint::Image::default(),
        release_type: card.release_type.into(),
        source: card.source.into(),
        quality_detail: card.quality_detail.into(),
        track_count: card.track_count.into(),
        plain_year: card.plain_year.into(),
        removing: false,
        selected: false,
    }
}

/// Convert one `PlaylistCardData` into the Slint `SearchPlaylistItem`,
/// single-cover shape (slot 0 only). Mirrors label.rs's playlist converter:
/// no subtitle (1:1 with Tauri's PlaylistCardLite), cover-count 0 when there
/// is no artwork so the card draws its placeholder.
pub(crate) fn playlist_to_item(p: &PlaylistCardData) -> SearchPlaylistItem {
    SearchPlaylistItem {
        id: p.id.clone().into(),
        title: p.title.clone().into(),
        subtitle: "".into(),
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_playlist_row_pinned when a pin toggles anywhere).
        is_pinned: crate::pinned::is_pinned("playlist", &p.id),
        cover_count: if p.artwork_url.is_empty() { 0 } else { 1 },
        url1: p.artwork_url.clone().into(),
        url2: "".into(),
        url3: "".into(),
        url4: "".into(),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
        category: p.category.clone().into(),
        // Neutral dark letterbox until the cover decodes and the artwork
        // pipeline writes the real dominant colour (mirrors immersive::
        // dominant_cover_color's own fallback).
        dominant_color: slint::Color::from_rgb_u8(30, 30, 34),
        // Discover playlists are editorial (foreign Qobuz) → follow + copy.
        is_owned: false,
        is_following: false,
        is_copied: false,
    }
}
