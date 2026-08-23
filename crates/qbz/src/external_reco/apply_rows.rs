//! Row -> Slint item mappers shared by the artist/track/album appliers.

use qbz_external_reco::{AlbumReco, ArtistReco, TrackReco};

use crate::{AlbumCardItem, SlimItem};

pub(super) fn slim_from_artist(a: &ArtistReco) -> SlimItem {
    let id = a.qobuz_artist_id.to_string();
    SlimItem {
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_artist_row_pinned when a pin toggles anywhere). First:
        // it must borrow `id` before the `id:` initializer moves it.
        is_pinned: crate::pinned::is_pinned("artist", &id),
        id: id.into(),
        title: a.name.clone().into(),
        subtitle: a.subtitle.clone().into(),
        rank: "".into(),
        artwork_url: a.image_url.clone().into(),
        artwork: slint::Image::default(),
        // Live follow state from the login-seeded fav cache (the
        // pinned_section precedent) — a hardcoded `false` mislabeled
        // already-followed artists. Kept live afterwards by
        // search::mark_artist_followed on every toggle.
        following: crate::fav_cache::is_artist_favorite(a.qobuz_artist_id),
    }
}
pub(super) fn slim_from_track(t: &TrackReco) -> SlimItem {
    SlimItem {
        id: t.qobuz_track_id.to_string().into(),
        title: t.title.clone().into(),
        subtitle: t.artist.clone().into(),
        rank: "".into(),
        artwork_url: t.artwork_url.clone().into(),
        artwork: slint::Image::default(),
        following: false,
        // Track slims render pin-less rows — tracks are not pinnable.
        is_pinned: false,
    }
}
pub(crate) fn album_card(a: &AlbumReco) -> AlbumCardItem {
    AlbumCardItem {
        plays: 0,
        // Favorite heart state from the login-seeded cache (kept live by
        // main::set_album_row_favorite when a favorite toggles anywhere).
        is_favorite: crate::fav_cache::is_album_favorite(&a.qobuz_album_id),
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_album_row_pinned when a pin toggles anywhere).
        is_pinned: crate::pinned::is_pinned("album", &a.qobuz_album_id),
        id: a.qobuz_album_id.clone().into(),
        title: a.title.clone().into(),
        artist: a.artist.clone().into(),
        artist_id: a.artist_id.clone().into(),
        genre: "".into(),
        year: a.year.clone().into(),
        quality_tier: a.quality_tier.clone().into(),
        quality_label: a.quality_label.clone().into(),
        ribbon: "".into(),
        ribbon_kind: "".into(),
        artwork_url: a.artwork_url.clone().into(),
        artwork: slint::Image::default(),
        ..Default::default()
    }
}
