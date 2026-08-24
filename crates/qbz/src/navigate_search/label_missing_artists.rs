use crate::*;

// Artists whose /label/page entry carried no parseable image — resolve each
// missing image (custom portrait first, then v2/artist/get) and feed it
// through the same LabelArtist pipeline (model indices are stable once
// apply runs). Split out of `navigate_label` (navigate_search/part4.rs) to
// stay under the 130-line file cap; spawns its own task, matching the
// original inline `tokio::spawn`.
pub(crate) fn spawn_missing_label_artist_images(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    missing_artists: Vec<(usize, String, String)>,
) {
    tokio::spawn(async move {
        let mut net_jobs = Vec::new();
        let mut local_jobs = Vec::new();
        for (index, artist_id, name) in missing_artists {
            // A user-set custom portrait always wins (same rule as the
            // artist page).
            if let Some(path) = crate::custom_artwork::artist_image(&name) {
                local_jobs.push(artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::LabelArtist { index },
                    url: path,
                });
                continue;
            }
            if let Ok(id) = artist_id.parse::<u64>() {
                match runtime.core().get_artist(id).await {
                    Ok(artist) => {
                        if let Some(url) = artist.image.as_ref().and_then(|i| i.best()) {
                            net_jobs.push(artwork::ArtworkJob {
                                target: artwork::ArtworkTarget::LabelArtist { index },
                                url: url.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        log::debug!(
                            "[qbz-slint] label artist image fallback for {artist_id} failed: {e}"
                        );
                    }
                }
            }
        }
        artwork::spawn_loads(net_jobs, weak.clone(), image_cache.clone());
        artwork::spawn_local_loads(local_jobs, weak, image_cache);
    });
}
