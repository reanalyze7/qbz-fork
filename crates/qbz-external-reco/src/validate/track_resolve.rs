//! Live track resolution: ISRC search, MusicBrainz ISRC fallback, fuzzy match.

use qbz_integrations::MusicBrainzClient;
use qbz_models::Track;

use crate::matching::{select_best_match, MatchInput, MIN_SCORE};
use crate::types::{RecoSource, TrackCandidate, TrackReco};
use crate::RecoCatalog;

pub(super) fn build_track_reco(track: &Track, source: RecoSource) -> TrackReco {
    TrackReco {
        qobuz_track_id: track.id,
        title: track.title.clone(),
        artist: track
            .performer
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_default(),
        artwork_url: track
            .album
            .as_ref()
            .and_then(|al| al.image.best().cloned())
            .unwrap_or_default(),
        source,
    }
}

async fn find_by_isrc(catalog: &dyn RecoCatalog, isrc: &str) -> Option<Track> {
    let results = catalog.search_tracks(isrc, 5).await;
    results.into_iter().find(|t| {
        t.streamable
            && t.isrc
                .as_deref()
                .map(|c| c.eq_ignore_ascii_case(isrc))
                .unwrap_or(false)
    })
}

pub(super) async fn resolve_track_live(
    catalog: &dyn RecoCatalog,
    mb: &MusicBrainzClient,
    cand: &TrackCandidate,
    skip_mb: bool,
) -> Option<TrackReco> {
    if let Some(isrc) = cand.isrc.as_deref().filter(|s| !s.is_empty()) {
        if let Some(track) = find_by_isrc(catalog, isrc).await {
            return Some(build_track_reco(&track, cand.source));
        }
    }
    // The MusicBrainz recording->ISRC lookup is gated behind a SERIAL 1.1s/req
    // rate limiter; for a 50-track weekly playlist (×2 rows) that is ~110s of
    // pure waiting, so the row never paints in practice. `skip_mb` bypasses it
    // and relies on the fuzzy Qobuz search below — reliable for the mainstream
    // tracks these playlists contain. (This is THE reason the Weekly rows did
    // not appear while Fresh Releases — album-based, no MusicBrainz — did.)
    if !skip_mb {
        if let Some(mbid) = cand.recording_mbid.as_deref().filter(|s| !s.is_empty()) {
            let isrcs = mb.get_recording_isrcs(mbid).await.unwrap_or_default();
            for isrc in isrcs {
                if let Some(track) = find_by_isrc(catalog, &isrc).await {
                    return Some(build_track_reco(&track, cand.source));
                }
            }
        }
    }
    let query = format!("{} {}", cand.artist, cand.title);
    let candidates = catalog.search_tracks(query.trim(), 20).await;
    let input = MatchInput {
        artist: &cand.artist,
        title: &cand.title,
        album: cand.album.as_deref(),
        duration_ms: cand.duration_ms,
        isrc: cand.isrc.as_deref(),
    };
    let (best, score) = select_best_match(&input, &candidates);
    match best {
        Some(track) if score >= MIN_SCORE => Some(build_track_reco(track, cand.source)),
        _ => None,
    }
}
