//! Seeded similar albums (album page: similar to THIS album).

use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};

use crate::matching::normalize;
use crate::types::{AlbumCandidate, AlbumReco, RecoSource};
use crate::validate::is_slop;
use crate::RecoInputs;

use super::validate_pools::validate_album_pool;
use super::{album_key, rotate_take, DISPLAY_CAP};

/// "Albums similar to this one" for the album page. There is no Last.fm
/// album-similarity endpoint, so we replicate it from artist similarity:
/// `seed_artist` -> artist.getSimilar -> one top (not-slop, not-excluded) album
/// per similar artist -> resolve to the Qobuz catalog. `exclude_pairs` are the
/// (artist, title) of albums already shown by the Qobuz `/album/suggest` row,
/// so the two carousels don't overlap. Empty when Last.fm is not connected.
pub async fn build_similar_albums_seeded(
    inputs: &RecoInputs<'_>,
    seed_artist: &str,
    exclude_pairs: &[(String, String)],
) -> Vec<AlbumReco> {
    let Some(lf) = &inputs.lastfm else {
        return Vec::new();
    };
    if seed_artist.trim().is_empty() {
        return Vec::new();
    }
    let exclude_keys: HashSet<String> =
        exclude_pairs.iter().map(|(a, t)| album_key(a, t)).collect();
    let seed_key = normalize(seed_artist);

    let sims = lf
        .client
        .get_similar_artists(seed_artist, 30)
        .await
        .unwrap_or_default();

    let per_artist: Vec<(String, Vec<qbz_integrations::lastfm::LastFmAlbum>)> =
        stream::iter(sims.into_iter().map(|s| {
            let lf = lf;
            async move {
                let albums = lf.client.get_artist_top_albums(&s.name, 6).await.unwrap_or_default();
                (s.name, albums)
            }
        }))
        .buffered(4)
        .collect()
        .await;

    let mut seen: HashSet<String> = HashSet::new();
    let mut candidates: Vec<AlbumCandidate> = Vec::new();
    for (artist, albums) in per_artist {
        // Skip a similar artist that IS the seed (self-similarity edge case).
        if normalize(&artist) == seed_key {
            continue;
        }
        for al in albums {
            let k = album_key(&al.artist, &al.name);
            if exclude_keys.contains(&k) || is_slop(&al.artist, &al.name) || !seen.insert(k) {
                continue;
            }
            candidates.push(AlbumCandidate {
                artist: al.artist.clone(),
                title: al.name,
                upc: None,
                source: RecoSource::LastFm,
                score: al.playcount as f32,
                subtitle: format!("Similar to {artist}"),
            });
            // One album per similar artist, so the row spans many artists.
            break;
        }
    }
    candidates.truncate(40);
    let pool = validate_album_pool(inputs.catalog, inputs.cache, candidates).await;
    rotate_take(pool, inputs.rotation_seed, DISPLAY_CAP)
}
