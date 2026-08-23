//! DailyQ/WeeklyQ seed derivation: listened-track ids + the analysis payload.

use std::collections::HashSet;

use qbz_app::settings::reco_store::HomeSeedLimits;
use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::TrackToAnalyse;

use super::load::favorite_tracks;

/// Even-spread sample of up to `n` ids across `ids` (Tauri's pickSpread):
/// stride through the list so the analysis seeds are not all clustered.
pub(super) fn pick_spread(ids: &[u64], n: usize) -> Vec<u64> {
    if ids.len() <= n {
        return ids.to_vec();
    }
    (0..n).map(|i| ids[i * ids.len() / n]).collect()
}

/// The DailyQ/WeeklyQ listened-track seed: recent QOBUZ plays + Qobuz
/// favorites, deduped, capped at 120 (mirrors Tauri's continueListening +
/// favorites merge). Local/ephemeral recents carry non-Qobuz ids and are
/// excluded; `qobuz_download` offline copies keep the real Qobuz id. A
/// recents-only seed is frequently empty for local-heavy users, so favorites
/// guarantee a non-empty seed.
///
/// Reco-backed (Slice b3): the reco store's scored continue-listening +
/// favorite track seeds, which reflect the trained taste model. Falls back to
/// the local recents + favorites derivation when reco is cold/disabled so the
/// mix never goes empty. The call site and the rest of the mix path are
/// unchanged.
pub(super) async fn mix_listened_seed_ids<A>(runtime: &AppRuntime<A>) -> Vec<u64>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    // Prefer reco's scored seeds. Use generous per-bucket limits (the home
    // rows use a smaller default) so the mix has enough material.
    let limits = HomeSeedLimits {
        recent_albums: 0,
        continue_tracks: 80,
        top_artists: 0,
        favorites: 80,
    };
    if let Some(seeds) = crate::reco::home_seeds(limits) {
        let mut out: Vec<u64> = Vec::new();
        let mut seen: HashSet<u64> = HashSet::new();
        for id in seeds
            .continue_listening_track_ids
            .into_iter()
            .chain(seeds.favorite_track_ids)
        {
            if seen.insert(id) {
                out.push(id);
            }
        }
        if !out.is_empty() {
            out.truncate(120);
            return out;
        }
    }
    // Fallback: recent QOBUZ plays + Qobuz favorites (local/ephemeral
    // recents carry non-Qobuz ids and are excluded).
    let mut seeds: Vec<u64> = crate::recently::load()
        .into_iter()
        .filter(|t| !matches!(t.source.as_str(), "local" | "ephemeral"))
        .filter_map(|t| t.id.parse::<u64>().ok())
        .collect();
    let mut seen: HashSet<u64> = seeds.iter().copied().collect();
    for fav in favorite_tracks(runtime).await {
        if seen.insert(fav.id) {
            seeds.push(fav.id);
        }
    }
    seeds.truncate(120);
    seeds
}

/// Resolve up to 9 spread seeds into the `track_to_analysed` payload (the
/// PRIMARY DailyQ/WeeklyQ path, Tauri buildSeeds): `get_track` each, extract
/// `{track_id, artist_id, genre_id, label_id}` (artist = performer, else
/// composer; missing ids default to 0), drop any with `artist_id == 0`.
pub(super) async fn build_tracks_to_analyse<A>(
    runtime: &AppRuntime<A>,
    seeds: &[u64],
) -> Vec<TrackToAnalyse>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let mut analysed = Vec::new();
    for id in pick_spread(seeds, 9) {
        let Ok(track) = runtime.core().get_track(id).await else {
            continue;
        };
        let artist_id = track
            .performer
            .as_ref()
            .map(|a| a.id)
            .or_else(|| track.composer.as_ref().map(|a| a.id))
            .unwrap_or(0);
        if artist_id == 0 {
            continue;
        }
        analysed.push(TrackToAnalyse {
            track_id: track.id,
            artist_id,
            genre_id: track
                .album
                .as_ref()
                .and_then(|a| a.genre.as_ref())
                .map(|g| g.id)
                .unwrap_or(0),
            label_id: track
                .album
                .as_ref()
                .and_then(|a| a.label.as_ref())
                .map(|l| l.id)
                .unwrap_or(0),
        });
    }
    analysed
}
