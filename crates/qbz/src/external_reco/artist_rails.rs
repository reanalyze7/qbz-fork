//! The shared choke point for the two Recommended-Artist rails: exclusion
//! filtering, cross-rail dedup, and overflow retention for the live "not
//! interested" backfill (`artist_dismiss.rs`).

use std::collections::HashSet;

use qbz_external_reco::{compose_artist_rails, ArtistReco, ARTIST_DISPLAY_CAP};

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::apply_artists_tracks::apply_artists;
use super::row_kinds::ArtistRow;
use super::ARTIST_OVERFLOW;

/// Fold the three per-user exclusion sources — followed artists, the app-wide
/// blacklist, and the reco-scoped "not interested" dismissals — into one id
/// set for the rail composition. Every source is independently fail-open (an
/// unbound/unreadable store simply contributes nothing).
pub(super) fn artist_exclusions() -> HashSet<u64> {
    let mut ids = crate::fav_cache::all_artists();
    ids.extend(crate::artist_blacklist::ids_snapshot());
    ids.extend(crate::reco_dismiss::ids_snapshot());
    ids
}

/// THE paint choke point for the two Recommended-Artist rails — the fresh
/// build AND the cached-blob paint both funnel through here. Composes the
/// visible rows from the validated pools AFTER exclusion filtering +
/// cross-rail dedup (common wins), takes the first ARTIST_DISPLAY_CAP
/// survivors per rail, and retains the rest as backfill overflow for the
/// "not interested" live replacement (compose_artist_rails does the split;
/// the full pools ride the results blob, so a cached paint backfills too).
pub(super) fn apply_artist_rails(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    common_pool: Vec<ArtistReco>,
    recent_pool: Vec<ArtistReco>,
) {
    let excluded = artist_exclusions();
    let (common, recent) =
        compose_artist_rails(common_pool, recent_pool, &excluded, ARTIST_DISPLAY_CAP);
    if let Ok(mut g) = ARTIST_OVERFLOW.lock() {
        g.0 = common.overflow;
        g.1 = recent.overflow;
    }
    apply_artists(weak, cache, common.visible, ArtistRow::RecArtistsCommon);
    apply_artists(weak, cache, recent.visible, ArtistRow::RecArtistsRecent);
}

/// Pop the first retained-overflow candidate for `which` rail that still
/// passes the LIVE exclusions (a follow/blacklist may have landed since the
/// paint) and is not already visible in either rail. Non-passing entries stay
/// pooled (they may become eligible again, e.g. after an un-follow).
pub(super) fn pop_backfill(
    which: ArtistRow,
    visible: &HashSet<String>,
    excluded: &HashSet<u64>,
) -> Option<ArtistReco> {
    let mut g = ARTIST_OVERFLOW.lock().ok()?;
    let pool = match which {
        ArtistRow::RecArtistsCommon => &mut g.0,
        ArtistRow::RecArtistsRecent => &mut g.1,
        ArtistRow::TopArtists => return None,
    };
    let idx = pool.iter().position(|r| {
        !excluded.contains(&r.qobuz_artist_id)
            && !visible.contains(&r.qobuz_artist_id.to_string())
    })?;
    Some(pool.remove(idx))
}
