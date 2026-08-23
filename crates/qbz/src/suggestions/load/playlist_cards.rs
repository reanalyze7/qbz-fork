//! Curated playlist-card assembly: the first N `artist.playlists`, each
//! resolved to a book collage of distinct album covers.

use std::collections::HashSet;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::super::covers::{track_album_cover, track_album_id};
use super::super::types::PlaylistCard;
use super::super::{BOOK_COVERS, MAX_PLAYLIST_CARDS};

/// Resolve the first [`MAX_PLAYLIST_CARDS`] curated playlists into
/// `PlaylistCard`s, fetching each full playlist to harvest up to
/// [`BOOK_COVERS`] distinct album covers (falling back to the playlist's own
/// image when no track covers are found).
pub(super) async fn build<A>(
    runtime: &AppRuntime<A>,
    artist: &qbz_models::Artist,
) -> Vec<PlaylistCard>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let mut playlist_cards: Vec<PlaylistCard> = Vec::new();
    let Some(playlists) = artist.playlists.as_ref() else {
        return playlist_cards;
    };
    for playlist in playlists.iter().take(MAX_PLAYLIST_CARDS) {
        // Fetch the full playlist to harvest up to BOOK_COVERS distinct
        // album covers.
        let mut cover_urls: Vec<String> = Vec::new();
        match runtime.core().get_playlist(playlist.id).await {
            Ok(full) => {
                if let Some(container) = full.tracks.as_ref() {
                    let mut seen_albums: HashSet<String> = HashSet::new();
                    for track in &container.items {
                        let (Some(url), Some(album_id)) =
                            (track_album_cover(track), track_album_id(track))
                        else {
                            continue;
                        };
                        if seen_albums.insert(album_id) {
                            cover_urls.push(url);
                            if cover_urls.len() >= BOOK_COVERS {
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "[qbz-slint] suggestions get_playlist({}) failed: {e}",
                    playlist.id
                );
            }
        }
        // Fallback to the playlist's own images when no track covers found.
        if cover_urls.is_empty() {
            if let Some(images) = playlist.images.as_ref() {
                if let Some(img) = images.iter().find(|s| !s.is_empty()) {
                    cover_urls.push(img.clone());
                }
            }
        }
        playlist_cards.push(PlaylistCard {
            id: playlist.id.to_string(),
            name: playlist.name.clone(),
            track_count: playlist.tracks_count,
            cover_urls,
        });
    }
    playlist_cards
}
