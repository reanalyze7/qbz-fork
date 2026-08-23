//! The instant-paint path: read the 48h results blob and, on a hit, paint
//! the non-weekly rows immediately then (re)build the two Weekly rows from
//! their own per-week cache.

use qbz_external_reco::{ExternalCarousels, RecoInputs};

use crate::artwork::ImageCache;
use crate::{AppWindow, ExternalRecoState};

use super::super::apply_all::{apply_all, build_and_apply_weeklies};

/// Try to paint the tab from the results cache. Returns `true` (and has
/// already painted + kicked off the Weekly rebuild) on a hit; `false` means
/// the caller must run the full build.
pub(super) async fn try_paint_cached(
    inputs: &RecoInputs<'_>,
    weak: &slint::Weak<AppWindow>,
    image_cache: &ImageCache,
    source_key: &str,
    ttl_secs: i64,
) -> bool {
    let Some(cache_mutex) = inputs.cache else {
        return false;
    };
    let cached = cache_mutex.lock().ok().and_then(|g| g.get_results(source_key, ttl_secs));
    let Some(json) = cached else {
        return false;
    };
    let Ok(result) = serde_json::from_str::<ExternalCarousels>(&json) else {
        return false;
    };

    apply_all(weak, image_cache, result);
    // The non-weekly rows painted instantly from the blob; the two Weekly
    // rows rebuild from their own per-week cache, so show their skeletons
    // until build_and_apply_weeklies fills them (instant on a weekly-cache
    // hit).
    if inputs.listenbrainz.is_some() {
        let w = weak.clone();
        let _ = w.upgrade_in_event_loop(|w| {
            let s = w.global::<ExternalRecoState>();
            s.set_pending_weekly_exploration(true);
            s.set_pending_weekly_jams(true);
        });
    }
    build_and_apply_weeklies(inputs, weak, image_cache).await;
    true
}
