use crate::*;

/// Open the LabelView landing — the rich label page (header + popular
/// tracks + releases/critics/playlists/artists/more-labels carousels).
/// Reached by clicking a label anywhere.
pub(crate) fn navigate_label(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    label_id: u64,
    name: String,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            label::reset_label_page(&w);
            w.global::<NavState>().set_view(ContentView::Label);
        });
        match label::load_label_page(&runtime, label_id, &name).await {
            Ok(payload) => {
                let jobs = label::page_artwork_jobs(&payload);
                let image_url = payload.image_url.clone();
                // Artists whose /label/page entry carried no parseable image —
                // left as "" the carousel keeps the placeholder forever.
                // Tauri-parity fallback (LabelView.svelte loadArtistImages):
                // resolve each missing image (custom portrait first, then
                // v2/artist/get) and feed it through the same LabelArtist
                // pipeline (model indices are stable once apply runs).
                let missing_artists: Vec<(usize, String, String)> = payload
                    .artists
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.image_url.is_empty())
                    .map(|(i, a)| (i, a.id.clone(), a.name.clone()))
                    .collect();
                // Catalog/library toggle: the per-label favorites index (seeded
                // at login from the same fetch as library_by_artist).
                let lib = crate::library_by_label::get(&label_id.to_string());
                // Cover jobs for the "In library" grid — the models seed with
                // empty images (same pattern as the artist library tab).
                let lib_jobs: Vec<artwork::ArtworkJob> = lib
                    .as_ref()
                    .map(|lib| {
                        lib.tracks
                            .iter()
                            .enumerate()
                            .filter(|(_, t)| !t.artwork_url.is_empty())
                            .map(|(index, t)| artwork::ArtworkJob {
                                target: artwork::ArtworkTarget::LabelLibraryTrack { index },
                                url: t.artwork_url.clone(),
                            })
                            .chain(
                                lib.albums
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, a)| !a.artwork_url.is_empty())
                                    .map(|(index, a)| artwork::ArtworkJob {
                                        target: artwork::ArtworkTarget::LabelLibraryAlbum { index },
                                        url: a.artwork_url.clone(),
                                    }),
                            )
                            .collect()
                    })
                    .unwrap_or_default();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    label::apply_label_page(&w, payload);
                    if let Some(lib) = lib.as_ref() {
                        label::apply_label_library(&w, lib);
                    }
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                artwork::spawn_loads(lib_jobs, weak.clone(), image_cache.clone());
                if !missing_artists.is_empty() {
                    spawn_missing_label_artist_images(
                        runtime.clone(),
                        weak.clone(),
                        image_cache.clone(),
                        missing_artists,
                    );
                }
                if !image_url.is_empty() {
                    if let Some((pixels, width, height)) =
                        artwork::fetch_and_decode(&image_url, &image_cache, 240).await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            label::apply_image(&w, &pixels, width, height);
                        });
                    }
                }
            }
            Err(e) => {
                log::error!("[qbz-slint] label page load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    let s = w.global::<LabelState>();
                    s.set_loading(false);
                    s.set_page_loaded(true);
                });
            }
        }
    });
}

