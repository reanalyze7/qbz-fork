//! Write the freshly-built result into the 48h results cache, guarding
//! against poisoning it with a transient ListenBrainz failure.

use std::sync::Mutex;

use qbz_external_reco::{ExternalCarousels, RecoCache};

/// Store the built result for instant future opens (48h TTL). GUARD against
/// poisoning the cache with a TRANSIENT ListenBrainz failure (rate-limit /
/// network / token-not-yet-restored): if LB is connected but EVERY
/// LB-sourced row (Weekly Exploration/Jams + Fresh Releases) came back
/// empty, skip the write so the next open re-fetches — otherwise the empty
/// result would hide those rows for the full 48h. (Owner-reported: the
/// Weeklys showed once, then vanished on restart.)
pub(super) fn write_results_cache(
    cache_mutex: Option<&Mutex<RecoCache>>,
    listenbrainz_connected: bool,
    source_key: &str,
    collector: &ExternalCarousels,
) {
    let Some(cache_mutex) = cache_mutex else {
        return;
    };
    let lb_all_empty = collector.weekly_exploration.is_empty()
        && collector.weekly_jams.is_empty()
        && collector.fresh_releases.is_empty();
    if listenbrainz_connected && lb_all_empty {
        log::warn!(
            "[reco] ListenBrainz connected but all LB rows empty — skipping \
             the results-cache write (likely transient; next open re-fetches)"
        );
        return;
    }
    if let Ok(json) = serde_json::to_string(collector) {
        if let Ok(guard) = cache_mutex.lock() {
            guard.put_results(source_key, &json);
        }
    }
}
