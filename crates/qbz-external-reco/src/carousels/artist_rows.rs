//! Recommended Artists — common (overall top) vs recent (1-month top).
//!
//! Split so a recent one-off binge (e.g. a soundtrack) can't pollute the core
//! taste row, and so the user gets more, better-targeted rows. Round-robin per
//! seed inside each row so no single seed floods the carousel.

use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};

use crate::matching::normalize;
use crate::types::{ArtistCandidate, ArtistReco, ExtHistory, RecoSource};
use crate::RecoInputs;

use super::validate_pools::validate_artist_pool;
use super::{rotate_take, ARTIST_SEEDS, SIMILAR_PER_SEED};

const PER_SEED_CAP: usize = 8;

/// Interleave per-seed candidate lists round-robin (fair representation).
fn round_robin<T>(groups: Vec<Vec<T>>) -> Vec<T> {
    let mut iters: Vec<std::vec::IntoIter<T>> = groups.into_iter().map(|g| g.into_iter()).collect();
    let mut out = Vec::new();
    loop {
        let mut any = false;
        for it in iters.iter_mut() {
            if let Some(x) = it.next() {
                out.push(x);
                any = true;
            }
        }
        if !any {
            break;
        }
    }
    out
}

async fn similar_artist_row(
    inputs: &RecoInputs<'_>,
    history: &ExtHistory,
    period: &str,
) -> Vec<ArtistReco> {
    let Some(lf) = &inputs.lastfm else {
        return Vec::new();
    };
    let seeds: Vec<String> = lf
        .client
        .get_top_artists(&lf.username, period, 12)
        .await
        .unwrap_or_default()
        .into_iter()
        .take(ARTIST_SEEDS)
        .map(|a| a.name)
        .collect();
    let seeds_norm: HashSet<String> = seeds.iter().map(|s| normalize(s)).collect();

    let sim_results: Vec<(String, Vec<qbz_integrations::lastfm::LastFmSimilarArtist>)> =
        stream::iter(seeds.into_iter().map(|seed| {
            let lf = lf;
            async move {
                let sims = lf
                    .client
                    .get_similar_artists(&seed, SIMILAR_PER_SEED)
                    .await
                    .unwrap_or_default();
                (seed, sims)
            }
        }))
        .buffered(4)
        .collect()
        .await;

    // One candidate list per seed, deduped globally (first seed wins), capped so
    // one seed cannot dominate; then round-robin interleaved.
    let mut seen_global: HashSet<String> = HashSet::new();
    let mut groups: Vec<Vec<ArtistCandidate>> = Vec::new();
    for (seed, sims) in sim_results {
        let mut list: Vec<ArtistCandidate> = Vec::new();
        for s in sims {
            let nk = normalize(&s.name);
            if nk.is_empty()
                || history.artist_names.contains(&nk)
                || seeds_norm.contains(&nk)
                || !seen_global.insert(nk)
            {
                continue;
            }
            list.push(ArtistCandidate {
                name: s.name,
                source: RecoSource::LastFm,
                score: s.match_score as f32,
                subtitle: format!("Similar to {}", seed),
            });
            if list.len() >= PER_SEED_CAP {
                break;
            }
        }
        groups.push(list);
    }
    let candidates: Vec<ArtistCandidate> = round_robin(groups).into_iter().take(45).collect();
    let pool = validate_artist_pool(inputs.catalog, inputs.cache, candidates).await;
    // Rotate for daily variety but do NOT truncate at DISPLAY_CAP: the paint
    // layer composes the visible rows AFTER followed/blacklisted/dismissed
    // filtering + cross-rail dedup (compose_artist_rails) and keeps the
    // remaining validated candidates as backfill overflow — they ride the
    // results blob, so replacements need no extra network.
    let take = pool.len();
    rotate_take(pool, inputs.rotation_seed, take)
}

/// "More like the artists you love" — your COMMON taste (overall top).
pub async fn build_rec_artists_common(
    inputs: &RecoInputs<'_>,
    history: &ExtHistory,
) -> Vec<ArtistReco> {
    similar_artist_row(inputs, history, "overall").await
}

/// "Based on what you've been into lately" — your RECENT taste (1-month top).
pub async fn build_rec_artists_recent(
    inputs: &RecoInputs<'_>,
    history: &ExtHistory,
) -> Vec<ArtistReco> {
    similar_artist_row(inputs, history, "1month").await
}
