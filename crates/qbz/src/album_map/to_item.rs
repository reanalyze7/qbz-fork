//! `to_item`: build the Slint `AlbumCardItem` from an `AlbumCard`.

use crate::AlbumCardItem;

use super::AlbumCard;

/// Build a Slint `AlbumCardItem` from an `AlbumCard`. SOURCE is left empty
/// (single-source Qobuz context — hide the column with `show-source: false`).
pub fn to_item(card: AlbumCard) -> AlbumCardItem {
    AlbumCardItem {
        plays: 0,
        // Favorite heart state. Local albums read the local-favorites
        // store (their composite keys never match a Qobuz favorite id);
        // Qobuz albums read the login-seeded fav cache so every card surface
        // renders the filled heart in sync with the album-detail header.
        is_favorite: if card.source == "local" {
            crate::local_favorites::is_favorite("album", &card.id)
        } else {
            crate::fav_cache::is_album_favorite(&card.id)
        },
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
        ribbon: "".into(),
        ribbon_kind: "".into(),
        artwork_url: card.artwork_url.into(),
        artwork: slint::Image::default(),
        // List-row extras — feed the AlbumListRow columns (TYPE / QUALITY /
        // TRACKS / YEAR) for the list view toggle.
        release_type: card.release_type.into(),
        source: card.source.into(),
        quality_detail: card.quality_detail.into(),
        track_count: card.track_count.into(),
        plain_year: card.plain_year.into(),
        ..Default::default()
    }
}
