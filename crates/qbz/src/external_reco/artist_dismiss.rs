//! Live "Not interested" dismissal for the two Recommended-Artist rails.
use slint::ComponentHandle;

use std::collections::HashSet;

use slint::{Model, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, ExternalRecoState, SlimItem};

use super::apply_rows::slim_from_artist;
use super::artist_rails::{artist_exclusions, pop_backfill};
use super::row_kinds::ArtistRow;

/// Live "Not interested": drop the artist card from the Recommended-Artist
/// rail that carries it and backfill the freed slot from that rail's retained
/// overflow — already Qobuz-validated, so no network. Runs on the Slint event
/// loop. Returns the (name, image_url) snapshot of the dismissed card (for the
/// dismissal store + toast), or None when the artist is not currently in a
/// reco rail (e.g. dismissed from a search/home card).
pub fn apply_artist_dismissal(
    w: &AppWindow,
    cache: &ImageCache,
    artist_id: &str,
) -> Option<(String, String)> {
    let s = w.global::<ExternalRecoState>();
    let common_model = s.get_rec_artists_common();
    let recent_model = s.get_rec_artists_recent();
    let common_rows: Vec<SlimItem> = (0..common_model.row_count())
        .filter_map(|i| common_model.row_data(i))
        .collect();
    let recent_rows: Vec<SlimItem> = (0..recent_model.row_count())
        .filter_map(|i| recent_model.row_data(i))
        .collect();
    let in_common = common_rows.iter().any(|it| it.id.as_str() == artist_id);
    let in_recent = recent_rows.iter().any(|it| it.id.as_str() == artist_id);
    if !in_common && !in_recent {
        return None;
    }
    let snapshot = common_rows
        .iter()
        .chain(recent_rows.iter())
        .find(|it| it.id.as_str() == artist_id)
        .map(|it| (it.title.to_string(), it.artwork_url.to_string()));

    let excluded = artist_exclusions();
    // Rebuild the affected models from the kept rows — their already-decoded
    // `artwork` images ride along, so only the backfilled row needs a job.
    let mut common_new: Vec<SlimItem> = common_rows
        .into_iter()
        .filter(|it| it.id.as_str() != artist_id)
        .collect();
    let mut recent_new: Vec<SlimItem> = recent_rows
        .into_iter()
        .filter(|it| it.id.as_str() != artist_id)
        .collect();
    // Visible ids across BOTH rails (post-removal): the backfill candidate
    // must not duplicate the other rail.
    let mut visible: HashSet<String> = common_new
        .iter()
        .chain(recent_new.iter())
        .map(|it| it.id.to_string())
        .collect();

    let mut jobs: Vec<ArtworkJob> = Vec::new();
    if in_common {
        if let Some(reco) = pop_backfill(ArtistRow::RecArtistsCommon, &visible, &excluded) {
            visible.insert(reco.qobuz_artist_id.to_string());
            if !reco.image_url.is_empty() {
                jobs.push(ArtworkJob {
                    url: reco.image_url.clone(),
                    target: ArtworkTarget::ExtRecoRecArtistCommon {
                        index: common_new.len(),
                    },
                });
            }
            common_new.push(slim_from_artist(&reco));
        }
        s.set_rec_artists_common(ModelRc::new(VecModel::from(common_new)));
    }
    if in_recent {
        if let Some(reco) = pop_backfill(ArtistRow::RecArtistsRecent, &visible, &excluded) {
            visible.insert(reco.qobuz_artist_id.to_string());
            if !reco.image_url.is_empty() {
                jobs.push(ArtworkJob {
                    url: reco.image_url.clone(),
                    target: ArtworkTarget::ExtRecoRecArtistRecent {
                        index: recent_new.len(),
                    },
                });
            }
            recent_new.push(slim_from_artist(&reco));
        }
        s.set_rec_artists_recent(ModelRc::new(VecModel::from(recent_new)));
    }
    if !jobs.is_empty() {
        crate::artwork::spawn_loads(jobs, w.as_weak(), cache.clone());
    }
    snapshot
}
