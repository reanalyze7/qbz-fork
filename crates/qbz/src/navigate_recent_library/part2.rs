use crate::*;

/// Open the full "Most Played Albums" page: load the ranked list, cache it,
/// render it. Local SQLite read, so no runtime + no error branch.
pub(crate) fn navigate_most_played_albums(
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            let s = w.global::<MostPlayedAlbumsState>();
            s.set_loading(true);
            s.set_search("".into());
            s.set_albums(slint::ModelRc::new(slint::VecModel::from(
                Vec::<AlbumCardItem>::new(),
            )));
            w.global::<NavState>()
                .set_view(ContentView::MostPlayedAlbums);
        });
        let rows = qbz_app::settings::album_play_history::all_albums();
        *MOST_PLAYED_ROWS.lock().unwrap() = rows.clone();
        apply_most_played_page(&weak, image_cache, rows);
    });
}

/// Re-filter the cached Most Played list by title/artist and re-render.
pub(crate) fn filter_most_played(
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    query: String,
) {
    let q = query.trim().to_lowercase();
    let rows: Vec<_> = MOST_PLAYED_ROWS
        .lock()
        .map(|guard| {
            guard
                .iter()
                .filter(|r| {
                    q.is_empty()
                        || r.title.to_lowercase().contains(&q)
                        || r.artist.to_lowercase().contains(&q)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    apply_most_played_page(&weak, image_cache, rows);
}

/// Set when a play lands in the recently-played store while the Home view is
/// not showing, so the next Home mount (`HomeActions.home-mounted`) re-reads
/// the LOCAL store into the rails. Cleared by `refresh_recent_rails`.
pub(crate) static RECENT_RAILS_DIRTY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Re-read the LOCAL recently-played store and re-push the Home "Recently
/// Played Tracks" / "Recently Played Albums" rails + their artwork. The rails
/// were previously read only inside the full discover load (`load_home`), so
/// plays during a session never surfaced until a restart — this is the
/// targeted refresh: a small local JSON read plus mostly cache-served artwork,
/// NO discover-index fetch. Mirrors `navigate_recent_albums`' off-UI-thread
/// read and its Qobuz vs local artwork split.
pub(crate) fn refresh_recent_rails(
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
    RECENT_RAILS_DIRTY.store(false, std::sync::atomic::Ordering::Relaxed);
    handle.spawn(async move {
        // Local file reads + blacklist filter — cheap, but keep them off the
        // UI thread like the sibling loaders.
        let recent = home::recent_track_slims();
        let cards = home::recent_album_cards();
        let mut jobs: Vec<artwork::ArtworkJob> = Vec::new();
        let mut local_jobs: Vec<artwork::ArtworkJob> = Vec::new();
        jobs.extend(recent.iter().enumerate().filter_map(|(idx, slim)| {
            (!slim.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                target: artwork::ArtworkTarget::Recent { idx },
                url: slim.artwork_url.clone(),
            })
        }));
        for (idx, card) in cards.iter().enumerate() {
            if card.artwork_url.is_empty() {
                continue;
            }
            let job = artwork::ArtworkJob {
                target: artwork::ArtworkTarget::RecentAlbum { idx },
                url: card.artwork_url.clone(),
            };
            if card.source == "local" {
                local_jobs.push(job);
            } else {
                jobs.push(job);
            }
        }
        let weak_for_local = weak.clone();
        let image_cache_local = image_cache.clone();
        let _ = weak.clone().upgrade_in_event_loop(move |w| {
            home::apply_recent_rails(&w, recent, cards);
        });
        artwork::spawn_loads(jobs, weak, image_cache);
        if !local_jobs.is_empty() {
            artwork::spawn_local_loads(local_jobs, weak_for_local, image_cache_local);
        }
    });
}

/// Base ("Recently added") order of the Home "Library Albums" rail
/// (`favoriteAlbums`, #566), cached on every home load so the header sort
/// dropdown can reorder without a re-fetch. Read by `apply_library_albums_sort`.
pub(crate) static LIB_ALBUMS_BASE: std::sync::Mutex<Vec<crate::foryou::AlbumCard>> =
    std::sync::Mutex::new(Vec::new());

