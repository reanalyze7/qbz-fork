use crate::*;

/// Open the full "Recently Played Albums" page (the Home rail's "View all").
/// LOCAL data: the play-history album store (crate::recently) mapped through
/// the same card funnel as the rail (`home::recent_album_cards` — blacklist
/// filter + date localization; `home::card_to_item` — is-favorite seeding),
/// so no runtime and no error branch (missing store = empty list). Artwork
/// splits Qobuz covers (plain loader) from local covers (source-aware
/// funnel), mirroring the rail's dispatch in `reload_home`.
pub(crate) fn navigate_recent_albums(
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            let s = w.global::<RecentAlbumsState>();
            s.set_loading(true);
            s.set_albums(slint::ModelRc::new(slint::VecModel::from(
                Vec::<AlbumCardItem>::new(),
            )));
            w.global::<NavState>().set_view(ContentView::RecentAlbums);
        });
        // Local file read + blacklist filter — cheap, but keep it off the UI
        // thread like the sibling loaders.
        let cards = home::recent_album_cards();
        let mut jobs: Vec<artwork::ArtworkJob> = Vec::new();
        let mut local_jobs: Vec<artwork::ArtworkJob> = Vec::new();
        for (idx, card) in cards.iter().enumerate() {
            if card.artwork_url.is_empty() {
                continue;
            }
            let job = artwork::ArtworkJob {
                target: artwork::ArtworkTarget::RecentAlbumsPage { idx },
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
            // card_to_item seeds is-favorite from the login cache — UI thread,
            // same as apply_home.
            let items: Vec<AlbumCardItem> =
                cards.into_iter().map(home::card_to_item).collect();
            let s = w.global::<RecentAlbumsState>();
            s.set_albums(slint::ModelRc::new(slint::VecModel::from(items)));
            s.set_loading(false);
        });
        artwork::spawn_loads(jobs, weak, image_cache);
        if !local_jobs.is_empty() {
            artwork::spawn_local_loads(local_jobs, weak_for_local, image_cache_local);
        }
    });
}

/// Full ranked list for the Most Played Albums View-all, cached so the search
/// box re-filters without re-querying SQLite (mirrors the Qobuz-playlists
/// filter). Written by the navigation, read by the filter.
pub(crate) static MOST_PLAYED_ROWS: std::sync::Mutex<
    Vec<qbz_app::settings::album_play_history::AlbumPlayRow>,
> = std::sync::Mutex::new(Vec::new());

/// Map a ranked row to a grid card, carrying its play count (drawn only here;
/// `plays` is 0 on every other AlbumCardItem).
pub(crate) fn most_played_item(row: &qbz_app::settings::album_play_history::AlbumPlayRow) -> AlbumCardItem {
    let mut item = home::card_to_item(crate::home::CardData {
        id: row.album_id.clone(),
        title: row.title.clone(),
        artist: row.artist.clone(),
        artist_id: row.artist_id.clone(),
        year: row.year.clone(),
        quality_tier: row.quality_tier.clone(),
        quality_label: row.quality_label.clone(),
        artwork_url: row.artwork_url.clone(),
        source: row.source.clone(),
        ..Default::default()
    });
    item.plays = row.plays as i32;
    item
}

/// Push ranked rows onto the Most Played Albums page + fire their artwork
/// (Qobuz plain loader vs local source-aware funnel, like the recent
/// page). Shared by the initial load and the search filter.
pub(crate) fn apply_most_played_page(
    weak: &slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    rows: Vec<qbz_app::settings::album_play_history::AlbumPlayRow>,
) {
    let mut jobs: Vec<artwork::ArtworkJob> = Vec::new();
    let mut local_jobs: Vec<artwork::ArtworkJob> = Vec::new();
    for (idx, row) in rows.iter().enumerate() {
        if row.artwork_url.is_empty() {
            continue;
        }
        let job = artwork::ArtworkJob {
            target: artwork::ArtworkTarget::MostPlayedAlbumsPage { idx },
            url: row.artwork_url.clone(),
        };
        if row.source == "local" {
            local_jobs.push(job);
        } else {
            jobs.push(job);
        }
    }
    let weak_for_local = weak.clone();
    let image_cache_local = image_cache.clone();
    let _ = weak.clone().upgrade_in_event_loop(move |w| {
        let items: Vec<AlbumCardItem> = rows.iter().map(most_played_item).collect();
        let s = w.global::<MostPlayedAlbumsState>();
        s.set_albums(slint::ModelRc::new(slint::VecModel::from(items)));
        s.set_loading(false);
    });
    artwork::spawn_loads(jobs, weak.clone(), image_cache);
    if !local_jobs.is_empty() {
        artwork::spawn_local_loads(local_jobs, weak_for_local, image_cache_local);
    }
}

