//! Favorite events (W3) — log ONLY a successful ADD (make==true && network
//! ok); the caller applies that gate. Logging an un-favorite or a failed add
//! would corrupt the taste signal. Favorited items are inherently Qobuz
//! catalog (the add_favorite API only succeeds for Qobuz ids), so no extra
//! source gate.

use qbz_app::settings::reco_store::{RecoEventInput, RecoEventType, RecoItemType};

use super::RECO;

/// Log a favorite of a Qobuz track.
pub fn log_favorite_track(track_id: u64, album_id: Option<String>, artist_id: Option<u64>) {
    if let Ok(guard) = RECO.lock() {
        if let Some(store) = guard.as_ref() {
            if let Err(e) = store.log_favorite_event(track_id, album_id, artist_id, None) {
                log::warn!("[reco] log_favorite_track failed: {e}");
            }
        }
    }
}

/// Log a favorite of a Qobuz album.
pub fn log_favorite_album(album_id: String, artist_id: Option<u64>) {
    insert_favorite(RecoItemType::Album, None, Some(album_id), artist_id);
}

/// Log a favorite of a Qobuz artist.
pub fn log_favorite_artist(artist_id: u64) {
    insert_favorite(RecoItemType::Artist, None, None, Some(artist_id));
}

fn insert_favorite(
    item_type: RecoItemType,
    track_id: Option<u64>,
    album_id: Option<String>,
    artist_id: Option<u64>,
) {
    if let Ok(guard) = RECO.lock() {
        if let Some(store) = guard.as_ref() {
            let ev = RecoEventInput {
                event_type: RecoEventType::Favorite,
                item_type,
                track_id,
                album_id,
                artist_id,
                playlist_id: None,
                genre_id: None,
            };
            if let Err(e) = store.insert_event(&ev) {
                log::warn!("[reco] log_favorite failed: {e}");
            }
        }
    }
}
