use crate::*;

/// Load an album and show the album view, then fetch its artwork. Shared
/// by the `open-album` callback and by history back/forward.
pub(crate) fn navigate_album(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    album_id: String,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            album::reset_album(&w);
            w.global::<NavState>().set_view(ContentView::Album);
        });
        match album::load_album(&runtime, &album_id).await {
            Ok(data) => {
                let artwork_url = data.artwork_url.clone();
                // Carousel inputs, captured before `data` is moved into apply.
                let carousel_artist_id = data.artist_id.clone();
                let carousel_artist_name = data.artist.clone();
                // A user-set custom cover (keyed by album id) wins and is the
                // only image source for albums with no Qobuz cover. Same bug
                // class as the artist portrait fix.
                let custom_cover_path = crate::custom_artwork::album_cover(&album_id);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    album::apply_album(&w, data);
                    w.global::<AlbumState>().set_loading(false);
                });

                // Polish carousels — "From the same artist" + "Listening
                // suggestions". Qobuz-only (this is the Qobuz album path; local
                // albums load through navigate_local_album), best-effort: each
                // failure hides its own carousel. Loaded after the album so the
                // tracklist paints first.
                {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let image_cache = image_cache.clone();
                    let album_id = album_id.clone();
                    tokio::spawn(async move {
                        let more = album::load_more_from_artist(
                            &runtime,
                            &carousel_artist_id,
                            &carousel_artist_name,
                            &album_id,
                        )
                        .await;
                        let image_cache_more = image_cache.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let jobs = album::apply_more_from_artist(&w, more);
                            artwork::spawn_loads(jobs, w.as_weak(), image_cache_more);
                        });

                        let suggestions = album::load_suggestions(&runtime, &album_id).await;
                        // Capture the Qobuz row's (artist, title) + ids before
                        // `suggestions` moves — the Last.fm row dedups against them.
                        let exclude_pairs: Vec<(String, String)> = suggestions
                            .cards
                            .iter()
                            .map(|c| (c.artist.clone(), c.title.clone()))
                            .collect();
                        let exclude_ids: std::collections::HashSet<String> =
                            suggestions.cards.iter().map(|c| c.id.clone()).collect();
                        {
                            let image_cache_sug = image_cache.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                let jobs = album::apply_suggestions(&w, suggestions);
                                artwork::spawn_loads(jobs, w.as_weak(), image_cache_sug);
                            });
                        }

                        // Second suggestions row, from Last.fm similar artists
                        // (only when Last.fm is connected). Best-effort: empty
                        // result hides the row. Deduped vs the Qobuz row, and
                        // the resolved row is cached per album for 30 days.
                        let lastfm_recos = crate::external_reco::load_similar_albums_seeded(
                            &runtime,
                            &album_id,
                            &carousel_artist_name,
                            &exclude_pairs,
                            &exclude_ids,
                        )
                        .await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let jobs = album::apply_lastfm_suggestions(&w, lastfm_recos);
                            artwork::spawn_loads(jobs, w.as_weak(), image_cache);
                        });
                    });
                }
                if let Some(path) = custom_cover_path {
                    if let Some((pixels, width, height)) = artwork::fetch_and_decode_ref(
                        &qbz_models::ArtworkRef::LocalFile(path),
                        &image_cache,
                        448,
                    )
                    .await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            album::apply_artwork(&w, &pixels, width, height);
                        });
                    }
                } else if !artwork_url.is_empty() {
                    if let Some((pixels, width, height)) =
                        artwork::fetch_and_decode(&artwork_url, &image_cache, 448).await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            album::apply_artwork(&w, &pixels, width, height);
                        });
                    }
                }
            }
            Err(e) => {
                log::error!("[qbz-slint] album load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<AlbumState>().set_loading(false);
                });
            }
        }
    });
}

