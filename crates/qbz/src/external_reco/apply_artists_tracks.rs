//! Per-row apply for the artist and track rails (models built on the UI
//! thread; `slint::Image` is `!Send`).
use slint::ComponentHandle;

use qbz_external_reco::{ArtistReco, TrackReco};
use slint::{ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, ExternalRecoState};

use super::apply_rows::{slim_from_artist, slim_from_track};
use super::row_kinds::{ArtistRow, TrackRow};

pub(super) fn apply_artists(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    rows: Vec<ArtistReco>,
    which: ArtistRow,
) {
    let jobs: Vec<ArtworkJob> = rows
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.image_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.image_url.clone(),
            target: match which {
                ArtistRow::RecArtistsCommon => ArtworkTarget::ExtRecoRecArtistCommon { index: i },
                ArtistRow::RecArtistsRecent => ArtworkTarget::ExtRecoRecArtistRecent { index: i },
                ArtistRow::TopArtists => ArtworkTarget::ExtRecoTopArtist { index: i },
            },
        })
        .collect();
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        let model = ModelRc::new(VecModel::from(
            rows.iter().map(slim_from_artist).collect::<Vec<_>>(),
        ));
        let s = w.global::<ExternalRecoState>();
        match which {
            ArtistRow::RecArtistsCommon => {
                s.set_rec_artists_common(model);
                s.set_pending_rec_artists_common(false);
            }
            ArtistRow::RecArtistsRecent => {
                s.set_rec_artists_recent(model);
                s.set_pending_rec_artists_recent(false);
            }
            ArtistRow::TopArtists => {
                s.set_top_artists(model);
                s.set_pending_top_artists(false);
            }
        }
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_tracks(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    rows: Vec<TrackReco>,
    which: TrackRow,
) {
    let jobs: Vec<ArtworkJob> = rows
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.artwork_url.is_empty())
        .map(|(i, t)| ArtworkJob {
            url: t.artwork_url.clone(),
            target: match which {
                TrackRow::WeeklyExploration => ArtworkTarget::ExtRecoWeeklyExploration { index: i },
                TrackRow::WeeklyJams => ArtworkTarget::ExtRecoWeeklyJams { index: i },
            },
        })
        .collect();
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        let model = ModelRc::new(VecModel::from(
            rows.iter().map(slim_from_track).collect::<Vec<_>>(),
        ));
        let s = w.global::<ExternalRecoState>();
        match which {
            TrackRow::WeeklyExploration => {
                s.set_weekly_exploration(model);
                s.set_pending_weekly_exploration(false);
            }
            TrackRow::WeeklyJams => {
                s.set_weekly_jams(model);
                s.set_pending_weekly_jams(false);
            }
        }
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}
