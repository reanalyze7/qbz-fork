//! Blocking artists + albums + custom-image load, merged into rows.

use crate::local_library::artists::merge::{merge_artists, ArtistRow};
use crate::local_library::artists::state::ARTIST_ALBUMS;
use crate::local_library::shared::exclude_network_folders_now;

/// Load + merge the artists master list off the UI thread. Also caches the
/// album set (for the right-pane filter) into `ARTIST_ALBUMS`.
pub(crate) fn load_and_merge_artists() -> Vec<ArtistRow> {
    // Same network flag as every browse tab: connectivity-keyed — see the
    // NETWORK-FOLDER VISIBILITY note.
    let exclude_network = exclude_network_folders_now();
    let artists =
        crate::library_db::with_db(|db| db.get_artists_with_filter(true, exclude_network))
            .unwrap_or_default();
    // Album cache for the right pane + album_count.
    let albums = crate::library_db::with_db(|db| {
        db.get_albums_with_full_filter(false, true, exclude_network)
    })
    .unwrap_or_default();
    // Seed custom AND previously-cached Qobuz portraits (fixes the
    // Tauri headline bug: its batch load command was never wired).
    let custom =
        crate::library_db::with_db(|db| db.get_all_artist_image_urls()).unwrap_or_default();
    let merged = merge_artists(artists, &albums, &custom);
    if let Ok(mut cache) = ARTIST_ALBUMS.lock() {
        *cache = albums;
    }
    merged
}
