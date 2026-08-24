// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for.
use crate::*;

/// Load an artist page and show the artist view, then fetch the portrait.
/// Shared by the `open-artist` callback and by history back/forward.
pub(crate) fn navigate_artist(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    artist_id: String,
) {
    let artist_id_for_state = artist_id.clone();
    handle.spawn(async move {
        let id_for_apply = artist_id_for_state.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            artist::reset_artist(&w);
            artist::reset_network_sidebar(&w);
            // Reflect whether THIS artist is blacklisted so the overflow
            // menu shows Show/Blacklist correctly and the hidden banner
            // appears. is_blacklisted auto-gates on the enabled flag (reads
            // false when the feature is disabled, which is acceptable here).
            let is_bl = crate::artist_blacklist::is_blacklisted_id_str(&id_for_apply);
            let st = w.global::<ArtistState>();
            // Seed the pin state from the pinned store (Home "Pinned"
            // section) — before set_id, which moves id_for_apply.
            st.set_pinned(crate::pinned::is_pinned("artist", &id_for_apply));
            st.set_id(id_for_apply.into());
            st.set_is_blacklisted(is_bl);
            w.global::<NavState>().set_view(ContentView::Artist);
        });
        match artist::load_artist(&runtime, &artist_id).await {
            Ok(data) => {
                let artwork_url = data.artwork_url.clone();
                let jobs = artist::artwork_jobs(&data);
                let artist_name = data.name.clone();
                // Resolve a user-set custom portrait (keyed by artist name)
                // up front — it persists across navigation and is the ONLY
                // image source for artists with no Qobuz portrait (Vicky).
                let custom_image_path = crate::custom_artwork::artist_image(&artist_name);
                let similar_names_for_discovery: Vec<String> = data
                    .similar_artists
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();
                // Catalog/library toggle: look up the per-artist index (favorites
                // this session) once, build cover jobs for the "In library" rows
                // (they seed with empty images — Slint can't fetch network art),
                // then seed the models + dispatch alongside the catalog jobs.
                let lib = crate::library_by_artist::get(&artist_id);
                let mut lib_jobs: Vec<artwork::ArtworkJob> = Vec::new();
                if let Some(lib) = lib.as_ref() {
                    for (index, t) in lib.tracks.iter().enumerate() {
                        if !t.artwork_url.is_empty() {
                            lib_jobs.push(artwork::ArtworkJob {
                                target: artwork::ArtworkTarget::ArtistLibraryTrack { index },
                                url: t.artwork_url.clone(),
                            });
                        }
                    }
                    for (index, a) in lib.albums.iter().enumerate() {
                        if !a.artwork_url.is_empty() {
                            lib_jobs.push(artwork::ArtworkJob {
                                target: artwork::ArtworkTarget::ArtistLibraryAlbum { index },
                                url: a.artwork_url.clone(),
                            });
                        }
                    }
                }
                let _ = weak.upgrade_in_event_loop(move |w| {
                    artist::apply_artist(&w, data);
                    if let Some(lib) = lib.as_ref() {
                        let ast = w.global::<ArtistState>();
                        ast.set_library_count(lib.count() as i32);
                        ast.set_library_tracks(crate::library_by_artist::track_items(&lib.tracks));
                        ast.set_library_albums(crate::library_by_artist::album_items(&lib.albums));
                    }
                    w.global::<ArtistState>().set_loading(false);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                artwork::spawn_loads(lib_jobs, weak.clone(), image_cache.clone());

                // Seed the follow heart: pull the user's followed artists (also
                // refreshes the in-memory cache the toggle reads), then reflect
                // whether THIS artist is followed.
                {
                    let runtime_fav = runtime.clone();
                    let weak_fav = weak.clone();
                    let aid_fav = artist_id.clone();
                    tokio::spawn(async move {
                        if let Ok(ids) = runtime_fav.core().favorite_artist_ids().await {
                            let is_fav = aid_fav
                                .parse::<u64>()
                                .map(|a| ids.contains(&a))
                                .unwrap_or(false);
                            crate::fav_cache::set_all_artists(ids);
                            let _ = weak_fav.upgrade_in_event_loop(move |w| {
                                let ast = w.global::<ArtistState>();
                                if ast.get_id().as_str() == aid_fav.as_str() {
                                    ast.set_is_following(is_fav);
                                }
                            });
                        }
                    });
                }

                // Magazine / Stories — fetch the editorial teasers in
                // parallel; the sidebar section stays hidden if there are none.
                {
                    let runtime_story = runtime.clone();
                    let weak_story = weak.clone();
                    let artist_id_story = artist_id.clone();
                    let image_cache_story = image_cache.clone();
                    tokio::spawn(async move {
                        let stories = artist::load_stories(&runtime_story, &artist_id_story).await;
                        let _ = weak_story.upgrade_in_event_loop(move |w| {
                            let jobs = artist::apply_stories(&w, stories);
                            artwork::spawn_loads(jobs, w.as_weak(), image_cache_story);
                        });
                    });
                }

                // Network sidebar — kick the MB enrichment off in
                // parallel with artwork. Origin shows a loading state
                // until the resolve + metadata calls return; on success
                // the resolved mbid is used to fetch relationships and
                // discovery candidates in sequence (the V2 cache, when
                // wired, will collapse repeat visits to a single shot).
                let runtime_mb = runtime.clone();
                let weak_mb = weak.clone();
                tokio::spawn(async move {
                    let _ = weak_mb.upgrade_in_event_loop(|w| {
                        let state = w.global::<NetworkSidebarState>();
                        state.set_origin_loading(true);
                        state.set_relationships_loading(true);
                        state.set_discovery_loading(true);
                    });
                    match artist::load_mb_metadata(&runtime_mb, &artist_name).await {
                        Ok(Some(meta)) => {
                            let mbid = meta.mbid.clone();
                            let _ = weak_mb.upgrade_in_event_loop(move |w| {
                                artist::apply_mb_metadata(&w, meta);
                            });
                            match artist::load_mb_relationships(&runtime_mb, &mbid).await {
                                Ok(data) => {
                                    let _ = weak_mb.upgrade_in_event_loop(move |w| {
                                        artist::apply_mb_relationships(&w, data);
                                    });
                                }
                                Err(e) => {
                                    log::warn!("[qbz-slint] MB relationships failed: {e}");
                                    let _ = weak_mb.upgrade_in_event_loop(|w| {
                                        w.global::<NetworkSidebarState>()
                                            .set_relationships_loading(false);
                                    });
                                }
                            }
                            match artist::load_mb_discovery(
                                &runtime_mb,
                                &mbid,
                                &artist_name,
                                similar_names_for_discovery,
                            )
                            .await
                            {
                                Ok(disc) => {
                                    let _ = weak_mb.upgrade_in_event_loop(move |w| {
                                        artist::apply_mb_discovery(&w, disc);
                                    });
                                }
                                Err(e) => {
                                    log::warn!("[qbz-slint] MB discovery failed: {e}");
                                    let _ = weak_mb.upgrade_in_event_loop(|w| {
                                        w.global::<NetworkSidebarState>()
                                            .set_discovery_loading(false);
                                    });
                                }
                            }
                        }
                        Ok(None) => {
                            let _ = weak_mb.upgrade_in_event_loop(|w| {
                                artist::apply_mb_unavailable(&w);
                            });
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] MB metadata load failed: {e}");
                            let _ = weak_mb.upgrade_in_event_loop(|w| {
                                artist::apply_mb_unavailable(&w);
                            });
                        }
                    }
                });

                if let Some(path) = custom_image_path {
                    // User-set custom portrait wins (and is the only source
                    // for artists with no Qobuz image).
                    if let Some((pixels, width, height)) = artwork::fetch_and_decode_ref(
                        &qbz_models::ArtworkRef::LocalFile(path),
                        &image_cache,
                        440,
                    )
                    .await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            artist::apply_artwork(&w, &pixels, width, height);
                        });
                    }
                } else if !artwork_url.is_empty() {
                    if let Some((pixels, width, height)) =
                        artwork::fetch_and_decode(&artwork_url, &image_cache, 440).await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            artist::apply_artwork(&w, &pixels, width, height);
                        });
                    }
                }
            }
            Err(e) => {
                log::error!("[qbz-slint] artist load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<ArtistState>().set_loading(false);
                });
            }
        }
    });
}


thread_local! {
    /// Debounce timer for the header live search — restarted on every
    /// keystroke, fires the search 300 ms after typing stops.
    static SEARCH_DEBOUNCE: slint::Timer = slint::Timer::default();

    /// Debounce timer for the cortinilla (live dropdown) network load —
    /// restarted on every keystroke so the skeleton shows while typing and a
    /// single clean result paint lands ~220 ms after typing stops (no cached
    /// instant-paint, so results never "jump" from a cached to a fresh state).
    static CORTINILLA_DEBOUNCE: slint::Timer = slint::Timer::default();

    /// Snapshot of the cortinilla payload currently shown, so a
    /// `cortinilla-row-clicked(flat_index)` can resolve the flat index back to
    /// the concrete row (kind/id/source) and dispatch. UI thread only; set
    /// whenever `apply_cortinilla` writes a new payload.
    static LAST_CORTINILLA: std::cell::RefCell<Option<search::CortinillaData>> =
        const { std::cell::RefCell::new(None) };

    /// Snapshot of the raw `LocalTrack` rows that backed the cortinilla's "On
    /// this device" section currently shown. The click router resolves a local
    /// row (`source == "local"`) to its concrete `LocalTrack` here (the row's
    /// `id` is the library row id) so it can play through
    /// `playback::play_local_tracks`. Updated in lockstep with `LAST_CORTINILLA`
    /// whenever a cortinilla payload is applied. UI thread only.
    static LAST_CORTINILLA_LOCAL: std::cell::RefCell<Vec<qbz_library::LocalTrack>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Stash for the "Duplicate tracks" confirm sub-modal. Slint can't hold a
    /// `Vec<u64>` ergonomically, so when a Qobuz→Qobuz add finds duplicates we
    /// park the full context here and the DuplicateConfirmActions handlers read
    /// it back. Cleared on add-all / add-new-only / cancel. The tuple is
    /// `(playlist_id, all_track_ids, duplicate_track_ids, playlist_name)`.
    static DUP_CONFIRM_STASH: std::cell::RefCell<
        Option<(u64, Vec<u64>, std::collections::HashSet<u64>, String)>
    > = const { std::cell::RefCell::new(None) };
}

