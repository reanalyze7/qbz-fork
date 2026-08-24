
use crate::search::rows::{AlbumRow, ArtistRow, PlaylistRow, TrackRowData};
use crate::{AlbumCardItem, SearchPlaylistItem, SlimItem, TrackItem};

pub(crate) fn album_item(row: AlbumRow) -> AlbumCardItem {
    AlbumCardItem {
        plays: 0,
        // Favorite heart state from the login-seeded cache (kept live by
        // main::set_album_row_favorite when a favorite toggles anywhere).
        is_favorite: crate::fav_cache::is_album_favorite(&row.id),
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_album_row_pinned when a pin toggles anywhere).
        is_pinned: crate::pinned::is_pinned("album", &row.id),
        id: row.id.into(),
        title: row.title.into(),
        artist: row.artist.into(),
        artist_id: row.artist_id.into(),
        genre: row.genre.into(),
        year: row.year.into(),
        quality_tier: row.quality_tier.into(),
        quality_label: row.quality_label.into(),
        ribbon: Default::default(),
        ribbon_kind: Default::default(),
        artwork_url: row.artwork_url.into(),
        artwork: slint::Image::default(),
        ..Default::default()
    }
}

pub(crate) fn track_item(row: TrackRowData) -> TrackItem {
    let is_favorite = crate::fav_cache::is_favorite(&row.id);
    let is_cached = crate::offline_cache::is_cached(&row.id);
    TrackItem {
        // Combined search DROPS blacklisted rows at build time (T4 snapshot
        // filter), so a row reaching here is never blacklisted (no greyout).
        is_blacklisted: false,
        id: row.id.into(),
        number: "".into(),
        title: row.title.into(),
        artist: row.artist.into(),
        album: "".into(),
        duration: row.duration.into(),
        quality_tier: row.quality_tier.into(),
        quality_detail: row.quality_detail.into(),
        explicit: row.explicit,
        selected: false,
        artwork_url: row.artwork_url.into(),
        artwork: slint::Image::default(),
        is_favorite,
        artist_id: row.artist_id.into(),
        album_id: row.album_id.into(),
        removing: false,
        cache_status: if is_cached { 3 } else { 0 },
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

pub(crate) fn artist_item(row: ArtistRow) -> SlimItem {
    SlimItem {
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_artist_row_pinned when a pin toggles anywhere). First:
        // it must borrow `row.id` before the `id:` initializer moves it.
        is_pinned: crate::pinned::is_pinned("artist", &row.id),
        id: row.id.into(),
        title: row.name.into(),
        subtitle: row.subtitle.into(),
        rank: Default::default(),
        artwork_url: row.artwork_url.into(),
        artwork: slint::Image::default(),
        following: row.following,
    }
}

pub(crate) fn playlist_item(row: PlaylistRow) -> SearchPlaylistItem {
    let url = |i: usize| -> slint::SharedString {
        row.cover_urls.get(i).cloned().unwrap_or_default().into()
    };
    SearchPlaylistItem {
        // Pin badge state from the per-user pinned store (kept live by
        // main::set_playlist_row_pinned when a pin toggles anywhere). First:
        // it must borrow `row.id` before the `id:` initializer moves it.
        is_pinned: crate::pinned::is_pinned("playlist", &row.id),
        id: row.id.into(),
        title: row.title.into(),
        subtitle: row.subtitle.into(),
        cover_count: row.cover_urls.len().min(4) as i32,
        url1: url(0),
        url2: url(1),
        url3: url(2),
        url4: url(3),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
        // Search playlist results carry no category subtag, and a transparent
        // dominant-colour is the sentinel for "no letterbox" — the collage
        // keeps the legacy cover-fit (contain + dominant colour is Discover-
        // only).
        category: "".into(),
        dominant_color: slint::Color::from_argb_u8(0, 0, 0, 0),
        is_owned: row.is_owned,
        is_following: row.is_following,
        is_copied: row.is_copied,
    }
}
