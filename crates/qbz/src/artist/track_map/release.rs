use qbz_models::PageArtistRelease;

use super::track::tier;
use crate::home::CardData;
use crate::AlbumCardItem;

pub(crate) fn map_release(release: PageArtistRelease) -> CardData {
    let artist = release
        .artist
        .map(|a| a.name.display)
        .or_else(|| release.artists.and_then(|list| list.into_iter().next().map(|a| a.name)))
        .unwrap_or_default();
    let artwork_url = release
        .image
        .and_then(|img| img.best().cloned())
        .unwrap_or_default();
    let year = release
        .dates
        .as_ref()
        .and_then(|d| d.original.as_deref())
        .and_then(|s| s.get(..4).map(|y| y.to_string()))
        .unwrap_or_default();
    let bit_depth = release.audio_info.as_ref().and_then(|a| a.maximum_bit_depth);
    let sample_rate = release
        .audio_info
        .as_ref()
        .and_then(|a| a.maximum_sampling_rate);
    let quality_tier = tier(bit_depth).to_string();
    let quality_label = match (bit_depth, sample_rate) {
        (Some(bd), Some(sr)) => format!("{}-bit / {} kHz", bd, sr),
        _ => String::new(),
    };
    // Per-release press award → gold ribbon (the AlbumCard "press" ribbon
    // is already styled). First award name wins.
    let (ribbon, ribbon_kind) = release
        .awards
        .as_ref()
        .and_then(|list| list.first())
        .map(|award| (award.name.clone(), "press".to_string()))
        .unwrap_or_default();
    CardData {
        id: release.id,
        title: crate::album_map::format_album_title(&release.title, release.version.as_deref()),
        artist,
        artist_id: String::new(),
        genre: release.genre.map(|g| g.name).unwrap_or_default(),
        year,
        quality_tier,
        quality_label,
        ribbon,
        ribbon_kind,
        artwork_url,
        ..CardData::default()
    }
}

/// On the artist page the card subtitle slot should show the
/// release year — the artist is redundant since we're already on
/// their page. The AlbumCard reads `artist` for its subtitle line,
/// so re-route year through that field instead of changing the
/// shared card primitive.
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
        artist: card.year.clone().into(),
        artist_id: "".into(),
        genre: card.genre.into(),
        plain_year: card.year.clone().into(),
        year: card.year.into(),
        quality_tier: card.quality_tier.into(),
        quality_label: card.quality_label.into(),
        ribbon: card.ribbon.into(),
        ribbon_kind: card.ribbon_kind.into(),
        artwork_url: card.artwork_url.into(),
        artwork: slint::Image::default(),
        ..Default::default()
    }
}
