//! The actual spawned load: build clients + `RecoInputs`, try the cached
//! paint, else run the full build, then maybe write the results cache.

use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;
use qbz_external_reco::{LastFmHandle, ListenBrainzHandle, LocalHistory, RecoCache, RecoInputs};
use qbz_integrations::{LastFmClient, ListenBrainzClient, MusicBrainzClient};

use crate::adapter::SlintAdapter;
use crate::artwork::ImageCache;
use crate::AppWindow;

use super::super::{rotation_seed, CoreRecoCatalog, CACHE_DIR};
use super::{latch_loaded, set_pending};

pub(super) fn spawn(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    force: bool,
) {
    handle.spawn(async move {
        let cfg = crate::scrobbler_settings::get();

        let lastfm_client = LastFmClient::new();
        let lb_client = ListenBrainzClient::new();
        if cfg.listenbrainz_is_authed() {
            lb_client
                .restore_token(cfg.listenbrainz_token.clone(), cfg.listenbrainz_username.clone())
                .await;
        }
        let mb_client = MusicBrainzClient::new();

        let lastfm = if cfg.lastfm_is_authed() && !cfg.lastfm_username.is_empty() {
            Some(LastFmHandle {
                username: cfg.lastfm_username.clone(),
                client: &lastfm_client,
            })
        } else {
            None
        };
        let listenbrainz = if cfg.listenbrainz_is_authed() && !cfg.listenbrainz_username.is_empty() {
            Some(ListenBrainzHandle {
                username: cfg.listenbrainz_username.clone(),
                client: &lb_client,
            })
        } else {
            None
        };

        let local = LocalHistory {
            known_artist_ids: crate::reco::known_artist_ids(2).unwrap_or_default(),
            ..Default::default()
        };

        let catalog = CoreRecoCatalog {
            runtime: runtime.clone(),
        };
        let cache_dir = CACHE_DIR.lock().ok().and_then(|g| g.clone());
        let cache = match &cache_dir {
            Some(dir) => match RecoCache::open_at(dir) {
                Ok(c) => Some(Mutex::new(c)),
                Err(e) => {
                    log::warn!("[reco] spawn: cache open failed ({e}) — running uncached");
                    None
                }
            },
            None => {
                log::warn!("[reco] spawn: cache dir not set (init_for_user not run?) — running uncached");
                None
            }
        };

        let inputs = RecoInputs {
            lastfm,
            listenbrainz,
            musicbrainz: &mb_client,
            catalog: &catalog,
            cache: cache.as_ref(),
            local,
            rotation_seed: rotation_seed(),
        };

        let source_key = format!(
            "results:lf={}:lb={}",
            inputs.lastfm.is_some(),
            inputs.listenbrainz.is_some()
        );
        log::info!(
            "[reco] spawn: lastfm={} listenbrainz={} source_key={source_key} force={force}",
            inputs.lastfm.is_some(),
            inputs.listenbrainz.is_some()
        );

        // Effective results-cache window (Recommendations setting -> seconds).
        let ttl_secs = crate::discover_prefs::reco_cache_ttl_secs();

        if !force
            && super::cache_paint::try_paint_cached(&inputs, &weak, &image_cache, &source_key, ttl_secs)
                .await
        {
            latch_loaded(&weak);
            return;
        }

        // Cache miss / stale: tell the user we're working, then build.
        crate::toast::info_weak(&weak, qbz_i18n::t("Generating recommendations…"));

        let cold_start = qbz_external_reco::is_cold_start(&inputs);
        {
            let w = weak.clone();
            let _ = w.upgrade_in_event_loop(move |w| {
                set_pending(&w, cold_start);
            });
        }

        let collector = super::build::build_full(&inputs, &weak, &image_cache, cold_start).await;

        super::cache_write::write_results_cache(
            inputs.cache,
            inputs.listenbrainz.is_some(),
            &source_key,
            &collector,
        );

        latch_loaded(&weak);
    });
}
