//! "From the same artist" carousel — other releases by this album's primary
//! artist, with the current album removed.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::{ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AlbumCardItem, AlbumState, AppWindow, DiscoverSection};

/// "From the same artist" carousel data. `Send` (plain cards).
pub struct MoreFromArtist {
    pub cards: Vec<crate::home::CardData>,
    /// Whether the carousel should show (non-empty + not Various Artists).
    pub show: bool,
}

/// Maximum cards in the "From the same artist" carousel.
const MORE_FROM_ARTIST_MAX: usize = 16;

/// Fetch + map other releases by `artist_id`, excluding `current_album_id`.
/// Skipped (returns hidden) when the artist id is missing/unparseable or the
/// artist is "Various Artists". Best-effort: a fetch error yields a hidden
/// carousel, never an error to the caller.
pub async fn load_more_from_artist<A>(
    runtime: &Arc<AppRuntime<A>>,
    artist_id: &str,
    artist_name: &str,
    current_album_id: &str,
) -> MoreFromArtist
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let hidden = MoreFromArtist {
        cards: Vec::new(),
        show: false,
    };
    if artist_name.eq_ignore_ascii_case("Various Artists") {
        return hidden;
    }
    let Ok(id) = artist_id.parse::<u64>() else {
        return hidden;
    };
    let resp = match runtime
        .core()
        .get_releases_grid(id, "album", 20, 0, Some("release_date"))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[qbz-slint] more-from-artist load failed: {e}");
            return hidden;
        }
    };
    let cards: Vec<crate::home::CardData> = resp
        .items
        .into_iter()
        .map(crate::artist::map_release)
        .filter(|c| c.id != current_album_id)
        .filter(|c| !crate::artist_blacklist::card_blacklisted(&c.id, &c.artist_id))
        .take(MORE_FROM_ARTIST_MAX)
        .collect();
    let show = !cards.is_empty();
    MoreFromArtist { cards, show }
}

/// Apply the "From the same artist" carousel. Runs on the Slint event loop.
/// Returns the artwork jobs for its cards.
pub fn apply_more_from_artist(window: &AppWindow, data: MoreFromArtist) -> Vec<ArtworkJob> {
    use slint::ComponentHandle;
    let items: Vec<AlbumCardItem> = data
        .cards
        .iter()
        .cloned()
        .map(crate::home::card_to_item)
        .collect();
    let jobs = data
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.artwork_url.is_empty())
        .map(|(i, c)| ArtworkJob {
            url: c.artwork_url.clone(),
            target: ArtworkTarget::AlbumMoreFromArtist { index: i },
        })
        .collect();
    let section = DiscoverSection {
        title: qbz_i18n::t("From the same artist").into(),
        endpoint: "".into(),
        albums: ModelRc::new(VecModel::from(items)),
    };
    let state = window.global::<AlbumState>();
    state.set_more_from_artist(section);
    state.set_show_more_from_artist(data.show);
    jobs
}
