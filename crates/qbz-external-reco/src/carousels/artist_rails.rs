//! Display composition (filter + cross-rail dedup + backfill split).

use std::collections::HashSet;

use crate::types::ArtistReco;

/// One artist rail after display composition: the visible rows (the first
/// `cap` survivors) plus the retained overflow (the remaining survivors, in
/// pool order), which the paint layer keeps for live backfill after a
/// "not interested" dismissal.
#[derive(Debug, Clone, Default)]
pub struct ArtistRailComposition {
    pub visible: Vec<ArtistReco>,
    pub overflow: Vec<ArtistReco>,
}

/// Compose the two Recommended-Artist rails for display:
///
/// - drop every candidate whose Qobuz id is in `excluded` (the frontend folds
///   followed / blacklisted / dismissed ids into that set; fail-soft = pass
///   an empty set),
/// - dedup WITHIN and ACROSS rails: an id shows in at most one rail — the
///   COMMON rail is composed first and wins — and overflow ids are claimed
///   too, so a common-overflow id cannot resurface in recent's visible,
/// - the first `cap` survivors per rail are visible; the rest is overflow.
pub fn compose_artist_rails(
    common_pool: Vec<ArtistReco>,
    recent_pool: Vec<ArtistReco>,
    excluded: &HashSet<u64>,
    cap: usize,
) -> (ArtistRailComposition, ArtistRailComposition) {
    let mut seen: HashSet<u64> = HashSet::new();
    let common = compose_one_rail(common_pool, excluded, &mut seen, cap);
    let recent = compose_one_rail(recent_pool, excluded, &mut seen, cap);
    (common, recent)
}

fn compose_one_rail(
    pool: Vec<ArtistReco>,
    excluded: &HashSet<u64>,
    seen: &mut HashSet<u64>,
    cap: usize,
) -> ArtistRailComposition {
    let mut out = ArtistRailComposition::default();
    for reco in pool {
        if excluded.contains(&reco.qobuz_artist_id) || !seen.insert(reco.qobuz_artist_id) {
            continue;
        }
        if out.visible.len() < cap {
            out.visible.push(reco);
        } else {
            out.overflow.push(reco);
        }
    }
    out
}
