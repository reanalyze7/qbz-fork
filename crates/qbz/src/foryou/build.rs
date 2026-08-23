//! Pure builders (no network) for the recently-played + top-artist
//! sections, plus the async wrapper Home (#566) shares with For You.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Artist;

use super::fetch::fetch_fav_artists;
use super::{AlbumCard, ArtistSlim};

pub(super) fn recent_album_cards(list: &[crate::recently::RecentAlbum]) -> Vec<AlbumCard> {
    list.iter()
        .cloned()
        .map(|a| AlbumCard {
            id: a.id,
            title: a.title,
            artist: a.artist,
            artist_id: String::new(),
            year: String::new(),
            quality_tier: a.quality_tier,
            quality_label: a.quality_label,
            artwork_url: a.artwork_url,
        })
        .collect()
}

pub(super) fn recent_track_slims() -> Vec<super::TrackSlim> {
    crate::recently::load()
        .into_iter()
        .take(24)
        .map(|t| super::TrackSlim {
            id: t.id,
            title: t.title,
            subtitle: t.subtitle,
            artwork_url: t.artwork_url,
        })
        .collect()
}

pub(super) fn top_artist_slims(fav_artists: &[Artist]) -> Vec<ArtistSlim> {
    fav_artists
        .iter()
        .take(18)
        .cloned()
        .map(|a| super::models::map_artist(a, true))
        .collect()
}

/// The full "Your Top Artists" data pipeline (favorite artists -> cap 18,
/// following=true) as one call. Shared with Home (#566), mirroring
/// [`super::build_albums::favorite_album_cards`]: For You's own
/// `artists_branch` keeps the pieces because it also needs the raw
/// `Vec<Artist>` for To-Follow / Spotlight.
pub(crate) async fn top_artist_cards<A>(runtime: &Arc<AppRuntime<A>>) -> Vec<ArtistSlim>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    top_artist_slims(&fetch_fav_artists(runtime).await)
}
