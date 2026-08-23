//! Recently-played rails, sourced from the local play-history store (no
//! network). Shared by [`super::load_home`] and the targeted recent-rails
//! refresh in `main.rs`.

use crate::home::{CardData, SlimData};

/// The recently-played TRACK window mapped to slim-card data, newest first,
/// capped at 24 (two carousel pages of 12 — the slim carousel does not show
/// beyond that). Local file read — cheap.
pub fn recent_track_slims() -> Vec<SlimData> {
    crate::recently::load()
        .into_iter()
        .take(24)
        .map(|track| SlimData {
            id: track.id,
            title: track.title,
            subtitle: track.subtitle,
            rank: String::new(),
            artwork_url: track.artwork_url,
        })
        .collect()
}

/// The recently-played album history mapped to card data, newest first.
/// Shared by the Home "Recently Played Albums" rail (via `load_home`) and
/// the full "View all" page (`main::navigate_recent_albums`), so both apply
/// the same blacklist filter and date localization. Local file read — cheap.
pub fn recent_album_cards() -> Vec<CardData> {
    crate::recently::load_albums()
        .into_iter()
        // Drop blocked albums (own id) at the SOURCE so the model + artwork jobs
        // stay index-aligned. Recently Played is Qobuz album ids.
        .filter(|album| !crate::artist_blacklist::is_album_blacklisted(&album.id))
        .map(|album| CardData {
            id: album.id,
            title: album.title,
            artist: album.artist,
            artist_id: String::new(),
            genre: album.genre,
            // Localize the stored ISO release date to "MMM D, YYYY" the
            // same way the discover cards do (empty stays empty).
            year: if album.release_date.is_empty() {
                String::new()
            } else {
                crate::dates::release_label(Some(&album.release_date))
            },
            quality_tier: album.quality_tier,
            quality_label: album.quality_label,
            ribbon: String::new(),
            ribbon_kind: String::new(),
            artwork_url: album.artwork_url,
            // Carry the origin so the card resolves source-aware artwork
            // (local file) and the play/open route correctly.
            source: album.source,
            // Recently-played cards render in the grid only — list-row
            // extras stay default.
            ..CardData::default()
        })
        .collect()
}
