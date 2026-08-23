use slint::{ModelRc, VecModel};

use crate::artist::data::ReleaseSection;
use crate::artist::track_map::card_to_item;
use crate::{AlbumCardItem, ArtistReleaseSection};

/// Map the raw `ReleaseSection`s into Slint `ArtistReleaseSection`s: apply the
/// persisted per-bucket sort, drop blacklisted albums, and translate the
/// (English) bucket title for display.
pub(crate) fn map_release_sections(sections: Vec<ReleaseSection>) -> Vec<ArtistReleaseSection> {
    sections
        .into_iter()
        .map(|section| {
            // Apply the persisted per-bucket sort up front so the first paint
            // already honors the user's choice.
            let sort = crate::artist_prefs::get_sort(&section.release_type);
            // Drop blocked albums (own id). The artist axis is moot here (you're
            // on the artist's own page) and CardData.artist_id is blank anyway.
            let mut albums: Vec<AlbumCardItem> = section
                .cards
                .into_iter()
                .filter(|c| !crate::artist_blacklist::card_blacklisted(&c.id, &c.artist_id))
                .map(card_to_item)
                .collect();
            crate::album_map::sort_album_items(&mut albums, &sort);
            ArtistReleaseSection {
                release_type: section.release_type.into(),
                // `section.title` is the English bucket title (kept English in
                // `map_artist` so jump-tab routing matches); translate for display.
                title: qbz_i18n::t(&section.title).into(),
                albums: ModelRc::new(VecModel::from(albums)),
                has_more: section.has_more,
                sort_by: sort.into(),
            }
        })
        .collect()
}
