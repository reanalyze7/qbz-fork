//! Recommended Albums (Last.fm: your artists' top albums, not scrobbled).

use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};

use crate::types::{AlbumCandidate, AlbumReco, ExtHistory, RecoSource};
use crate::validate::is_slop;
use crate::RecoInputs;

use super::validate_pools::validate_album_pool;
use super::{album_key, rotate_take, DISPLAY_CAP, KNOWN_ARTISTS_PER_BUILD};

pub async fn build_rec_albums(inputs: &RecoInputs<'_>, history: &ExtHistory) -> Vec<AlbumReco> {
    let Some(lf) = &inputs.lastfm else {
        return Vec::new();
    };
    // Lifetime top artists (not 1-month) for VOLUME: Recommended Albums shows
    // one album per artist, so we need many distinct artists to clear >=20
    // after Qobuz-catalog validation (the recent-taste rows cover 1-month).
    let artists: Vec<String> = lf
        .client
        .get_top_artists(&lf.username, "overall", 60)
        .await
        .unwrap_or_default()
        .into_iter()
        .take(KNOWN_ARTISTS_PER_BUILD)
        .map(|a| a.name)
        .collect();

    let per_artist: Vec<(String, Vec<qbz_integrations::lastfm::LastFmAlbum>)> =
        stream::iter(artists.into_iter().map(|name| {
            let lf = lf;
            async move {
                let albums = lf.client.get_artist_top_albums(&name, 6).await.unwrap_or_default();
                (name, albums)
            }
        }))
        .buffered(4)
        .collect()
        .await;

    let mut seen: HashSet<String> = HashSet::new();
    let mut candidates: Vec<AlbumCandidate> = Vec::new();
    for (artist, albums) in per_artist {
        for al in albums {
            let k = album_key(&al.artist, &al.name);
            if history.album_keys.contains(&k) || is_slop(&al.artist, &al.name) || !seen.insert(k) {
                continue;
            }
            candidates.push(AlbumCandidate {
                artist: al.artist.clone(),
                title: al.name,
                upc: None,
                source: RecoSource::LastFm,
                score: al.playcount as f32,
                subtitle: format!("From {} — you haven't heard this one", artist),
            });
            // One album per artist (owner request): take this artist's top
            // not-yet-heard album and move on, so the row spans >=20 artists.
            break;
        }
    }
    candidates.truncate(60);
    let pool = validate_album_pool(inputs.catalog, inputs.cache, candidates).await;
    rotate_take(pool, inputs.rotation_seed, DISPLAY_CAP)
}
