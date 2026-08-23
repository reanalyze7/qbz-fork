//! Pure data-shaping layer: Qobuz DTO -> Slint item.

use qbz_models::DiscoverPlaylist;

use crate::SearchPlaylistItem;

/// One loaded playlist: the shared single-cover card plus the list-row
/// subtitle (owner + track count) that the rail cards drop but the browse
/// list view renders.
pub(super) struct BrowseCard {
    pub(super) card: crate::home::PlaylistCardData,
    pub(super) subtitle: String,
}

/// Map a Discover playlist reusing the Home rail mapper, capturing the
/// owner + localized track count for the list rows before the payload
/// moves into `map_playlist` (same "owner   •   N tracks" convention as
/// the search playlist rows).
pub(super) fn map_browse(p: DiscoverPlaylist) -> BrowseCard {
    let mut subtitle = p.owner.name.clone();
    if p.tracks_count > 0 {
        let n = p.tracks_count;
        let tracks_label =
            qbz_i18n::tf("{} track", "{} tracks", n as i64, &[&n.to_string()]);
        if subtitle.is_empty() {
            subtitle = tracks_label;
        } else {
            subtitle = format!("{}   •   {}", subtitle, tracks_label);
        }
    }
    BrowseCard {
        card: crate::home::map_playlist(p),
        subtitle,
    }
}

/// Convert to the Slint item — the rail converter plus the subtitle (the
/// grid cards ignore it; the list rows render it).
pub(super) fn to_item(bc: &BrowseCard) -> SearchPlaylistItem {
    let mut item = crate::home::playlist_to_item(&bc.card);
    item.subtitle = bc.subtitle.clone().into();
    item
}
