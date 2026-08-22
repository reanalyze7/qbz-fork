//! Track resolution: ISRC -> MusicBrainz `inc=isrcs` -> Qobuz, else fuzzy text.

use qbz_integrations::MusicBrainzClient;

use crate::cache::CacheLookup;
use crate::matching::normalize;
use crate::types::{TrackCandidate, TrackReco};
use crate::RecoCatalog;

use super::track_resolve::resolve_track_live;
use super::Cache;

fn track_cache_key(c: &TrackCandidate) -> String {
    if let Some(isrc) = c.isrc.as_deref().filter(|s| !s.is_empty()) {
        format!("t:isrc:{}", isrc.to_uppercase())
    } else if let Some(mbid) = c.recording_mbid.as_deref().filter(|s| !s.is_empty()) {
        format!("t:mbid:{}", mbid)
    } else {
        format!("t:name:{}|{}", normalize(&c.artist), normalize(&c.title))
    }
}

/// Resolve a track candidate to a Qobuz track.
///
/// `skip_negative`: when true, the per-track NEGATIVE cache is ignored on BOTH
/// read and write. This is for ListenBrainz weekly playlists, whose resolution
/// runs under heavy concurrent first-build load: a transient throttle/timeout
/// returns `None`, and persisting that as a 7-day negative would lock the track
/// (and so the whole weekly row) out long after the hiccup passed — the exact
/// mechanism that made Weekly Exploration/Jams "vanish". POSITIVE hits are still
/// cached (cheap re-resolution), and the resolved set is cached per-week by the
/// caller.
pub async fn validate_track(
    catalog: &dyn RecoCatalog,
    mb: &MusicBrainzClient,
    cache: Cache<'_>,
    cand: &TrackCandidate,
    skip_negative: bool,
    skip_mb: bool,
) -> Option<TrackReco> {
    let key = track_cache_key(cand);
    if let Some(c) = cache {
        if let Ok(guard) = c.lock() {
            match guard.get(&key) {
                CacheLookup::Found(json) => {
                    if let Ok(mut reco) = serde_json::from_str::<TrackReco>(&json) {
                        reco.source = cand.source;
                        return Some(reco);
                    }
                }
                // A cached negative is authoritative only when we trust negatives.
                CacheLookup::Negative if !skip_negative => return None,
                _ => {}
            }
        }
    }
    let reco = resolve_track_live(catalog, mb, cand, skip_mb).await;
    if let Some(c) = cache {
        if let Ok(guard) = c.lock() {
            match &reco {
                Some(r) => guard.put(&key, "track", Some(&serde_json::to_string(r).unwrap_or_default())),
                // Only persist a negative when negatives are trusted (NOT for
                // weeklies — a transient miss must not stick for 7 days).
                None if !skip_negative => guard.put(&key, "track", None),
                None => {}
            }
        }
    }
    reco
}
