//! Pure data-shaping: Qobuz-domain -> favorites card structs.

mod artist_label;
mod track;

pub(crate) use artist_label::{map_artist, map_label};
pub(crate) use track::map_track;

#[derive(Clone)]
pub struct TrackCard {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    /// Composer id for the blacklist row stamp (D-FEAT: performer OR composer);
    /// "" when the track carries no composer.
    pub composer_id: String,
    pub album: String,
    pub album_id: String,
    pub genre: String,
    pub duration: String,
    pub quality_tier: String,
    pub quality_detail: String,
    pub explicit: bool,
    pub artwork_url: String,
    /// Qobuz label id from the nested album object ("" when the surface
    /// doesn't return it) — feeds the per-label library index behind the
    /// LabelPage catalog/library toggle. Not rendered by any card.
    pub label_id: String,
}

#[derive(Clone)]
pub struct ArtistCard {
    pub id: String,
    pub name: String,
    pub image_url: String,
}

#[derive(Clone)]
pub struct LabelCard {
    pub id: String,
    pub name: String,
    pub albums_line: String,
    pub image_url: String,
}
