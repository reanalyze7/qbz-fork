//! "Listening suggestions" carousel (`/album/suggest`) + the Last.fm
//! "similar albums" carousel that sits under it (same heading).

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AlbumCardItem, AlbumState, AppWindow, DiscoverSection};

/// "Listening suggestions" carousel data — albums similar to the open album.
/// `Send` (plain cards).
pub struct Suggestions {
    pub cards: Vec<crate::album_map::AlbumCard>,
    pub show: bool,
}

/// Fetch + map listening suggestions for `album_id`. Best-effort: an error or
/// an empty result yields a hidden carousel.
pub async fn load_suggestions<A>(runtime: &Arc<AppRuntime<A>>, album_id: &str) -> Suggestions
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let resp = match runtime.core().get_album_suggest(album_id).await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[qbz-slint] album suggestions load failed: {e}");
            return Suggestions {
                cards: Vec::new(),
                show: false,
            };
        }
    };
    let cards: Vec<crate::album_map::AlbumCard> = resp
        .albums
        .map(|page| page.items)
        .unwrap_or_default()
        .into_iter()
        .map(crate::album_map::map_album)
        .filter(|c| c.id != album_id)
        .filter(|c| !crate::artist_blacklist::card_blacklisted(&c.id, &c.artist_id))
        .collect();
    let show = !cards.is_empty();
    Suggestions { cards, show }
}

/// Apply the "Listening suggestions" carousel. Runs on the Slint event loop.
/// Returns the artwork jobs for its cards.
pub fn apply_suggestions(window: &AppWindow, data: Suggestions) -> Vec<ArtworkJob> {
    let items: Vec<AlbumCardItem> = data
        .cards
        .iter()
        .cloned()
        .map(crate::album_map::to_item)
        .collect();
    let jobs = data
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.artwork_url.is_empty())
        .map(|(i, c)| ArtworkJob {
            url: c.artwork_url.clone(),
            target: ArtworkTarget::AlbumSuggestion { index: i },
        })
        .collect();
    let section = DiscoverSection {
        title: qbz_i18n::t("Listening suggestions").into(),
        endpoint: "".into(),
        albums: ModelRc::new(VecModel::from(items)),
    };
    let state = window.global::<AlbumState>();
    state.set_suggestions_section(section);
    state.set_show_suggestions(data.show);
    jobs
}

/// Apply the Last.fm "similar albums" carousel (sits under the Qobuz
/// suggestions, same heading). Runs on the Slint event loop. Returns its
/// artwork jobs. `recos` is already deduped against the Qobuz row by the caller.
pub fn apply_lastfm_suggestions(
    window: &AppWindow,
    recos: Vec<qbz_external_reco::AlbumReco>,
) -> Vec<ArtworkJob> {
    let items: Vec<AlbumCardItem> = recos
        .iter()
        .map(crate::external_reco::album_card)
        .collect();
    let jobs = recos
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.artwork_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.artwork_url.clone(),
            target: ArtworkTarget::AlbumLastfmSuggestion { index: i },
        })
        .collect();
    let show = !items.is_empty();
    let section = DiscoverSection {
        title: "".into(),
        endpoint: "".into(),
        albums: ModelRc::new(VecModel::from(items)),
    };
    let state = window.global::<AlbumState>();
    state.set_lastfm_suggestions_section(section);
    state.set_show_lastfm_suggestions(show);
    jobs
}
