//! Shared external history gathering.

use std::collections::HashSet;

use crate::matching::normalize;
use crate::types::ExtHistory;
use crate::RecoInputs;

use super::{album_key, track_key};

pub async fn gather_history(inputs: &RecoInputs<'_>) -> ExtHistory {
    let mut artist_names = HashSet::new();
    let mut track_keys = HashSet::new();
    let mut album_keys = HashSet::new();

    if let Some(lf) = &inputs.lastfm {
        let (tops, recents, albums) = tokio::join!(
            lf.client.get_top_artists(&lf.username, "overall", 300),
            lf.client.get_recent_tracks(&lf.username, 200, 1),
            lf.client.get_user_top_albums(&lf.username, "overall", 300, 1),
        );
        for a in tops.unwrap_or_default() {
            artist_names.insert(normalize(&a.name));
        }
        for t in recents.unwrap_or_default() {
            artist_names.insert(normalize(&t.artist));
            track_keys.insert(track_key(&t.artist, &t.name));
        }
        for al in albums.unwrap_or_default() {
            artist_names.insert(normalize(&al.artist));
            album_keys.insert(album_key(&al.artist, &al.name));
        }
    }
    if let Some(lb) = &inputs.listenbrainz {
        let listens = lb.client.get_recent_listens(&lb.username, 1000).await.unwrap_or_default();
        for l in listens {
            artist_names.insert(normalize(&l.artist_name));
            track_keys.insert(track_key(&l.artist_name, &l.track_name));
        }
    }
    ExtHistory {
        artist_names,
        track_keys,
        album_keys,
    }
}
