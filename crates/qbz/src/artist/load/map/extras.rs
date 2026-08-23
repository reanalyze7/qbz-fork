use qbz_models::{PageArtistPlaylists, PageArtistSimilar};

use crate::artist::data::{PlaylistSlim, SimilarArtistData};

pub(crate) fn map_similar_artists(
    similar: Option<PageArtistSimilar>,
) -> Vec<SimilarArtistData> {
    similar
        .map(|s| {
            s.items
                .into_iter()
                .map(|item| SimilarArtistData {
                    id: item.id.to_string(),
                    name: item.name.display,
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn map_playlists(playlists: Option<PageArtistPlaylists>) -> Vec<PlaylistSlim> {
    playlists
        .map(|p| {
            p.items
                .into_iter()
                .map(|pl| {
                    let owner = pl
                        .owner
                        .and_then(|o| o.name)
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "Qobuz".to_string());
                    let track_count = pl.tracks_count.unwrap_or(0);
                    let image_url = pl
                        .images
                        .and_then(|imgs| imgs.rectangle)
                        .and_then(|rects| rects.into_iter().find(|s| !s.is_empty()))
                        .unwrap_or_default();
                    PlaylistSlim {
                        id: pl.id.to_string(),
                        title: pl.title.unwrap_or_default(),
                        subtitle: format!("{owner} · {track_count}"),
                        image_url,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
